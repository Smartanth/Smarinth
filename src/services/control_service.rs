use std::cell::RefCell;
use std::net::{IpAddr, SocketAddr, TcpListener};
use std::sync::{Arc, mpsc};
use std::thread;
use ntex::rt::System;
use ntex::server::ServerBuilder;
use ntex::service::fn_factory_with_config;
use ntex::{fn_service, ServiceFactory};

use ntex::time::Seconds;
use ntex::util::{ByteString, Ready};
use ntex_mqtt::v5;

use crate::configs::settings::Settings;
use crate::errors::control_error::ControlError;

#[derive(Debug, Clone)]
struct Session {
    client_id: String,
    subscriptions: RefCell<Vec<ByteString>>,
    sink: v5::MqttSink,
}

#[derive(Clone)]
pub struct ControlService {
    client: Arc<v5::client::Client>,
}

impl ControlService {
    pub async fn new(settings: &Arc<Settings>) -> Self {
        let ip_addr = settings.control.host.parse::<IpAddr>().unwrap();
        let address = SocketAddr::from((ip_addr, settings.control.port));

        if settings.control.embed {
            let _system = Self::create_broker(&address).await;
        }

        let client = v5::client::MqttConnector::new(address)
            .client_id(settings.control.client_id.clone())
            .keep_alive(Seconds(10))
            .connect()
            .await
            .unwrap();

        Self {
            client: Arc::new(client),
        }
    }

    async fn create_broker(address: &SocketAddr) -> System {
        let (tx, rx) = mpsc::channel();

        let listener = TcpListener::bind(&address).unwrap();

        thread::spawn(move || {
            let sys = System::new("Broker");

            tx.send(sys.system()).unwrap();
            sys.run(|| {
                ServerBuilder::new()
                    .listen("mqtt", listener, move |_| {
                        v5::MqttServer::new(Self::handshake)
                            .control(Self::control_service_factory())
                            .publish(fn_factory_with_config(|session: v5::Session<Session>| {
                                Ready::Ok::<_, ControlError>(fn_service(move |req| {
                                    Self::publish(session.clone(), req)
                                }))
                            }))
                            .finish()
                    })?
                    .workers(1)
                    .disable_signals()
                    .run();
                Ok(())
            })
        });

        rx.recv().unwrap()
    }

    async fn handshake(
        handshake: v5::Handshake
    ) -> Result<v5::HandshakeAck<Session>, ControlError> {
        tracing::debug!("Start Mqtt connection: {:?}", handshake);

        let session = Session {
            client_id: handshake.packet().client_id.to_string(),
            subscriptions: RefCell::new(Vec::new()),
            sink: handshake.sink(),
        };

        Ok(handshake.ack(session))
    }

    async fn publish(
        session: v5::Session<Session>,
        publish: v5::Publish
    ) -> Result<v5::PublishAck, ControlError> {
        tracing::debug!(
            "incoming client publish ({:?}): {:?} -> {:?}",
            session.state(),
            publish.id(),
            publish.topic()
        );

        if session.subscriptions.borrow().contains(&publish.packet().topic) {
            session
                .sink
                .publish(publish.packet().topic.clone(), publish.packet().payload.clone())
                .send_at_least_once()
                .await
                .unwrap();
        }

        Ok(publish.ack())
    }

    fn control_service_factory() -> impl ServiceFactory<
        v5::ControlMessage<ControlError>,
        v5::Session<Session>,
        Response = v5::ControlResult,
        Error = ControlError,
        InitError = ControlError,
    > {
        fn_factory_with_config(|session: v5::Session<Session>| {
            Ready::Ok(fn_service(move |control| match control {
                v5::ControlMessage::Auth(a) => Ready::Ok(a.ack(v5::codec::Auth::default())),
                v5::ControlMessage::Error(e) => {
                    Ready::Ok(e.ack(v5::codec::DisconnectReasonCode::UnspecifiedError))
                }
                v5::ControlMessage::ProtocolError(e) => Ready::Ok(e.ack()),
                v5::ControlMessage::Ping(p) => Ready::Ok(p.ack()),
                v5::ControlMessage::Disconnect(d) => Ready::Ok(d.ack()),
                v5::ControlMessage::Subscribe(mut s) => {
                    s.iter_mut().for_each(|mut s| {
                        session.subscriptions.borrow_mut().push(s.topic().clone());
                        s.confirm(v5::QoS::AtLeastOnce);
                    });

                    Ready::Ok(s.ack())
                }
                v5::ControlMessage::Unsubscribe(s) => Ready::Ok(s.ack()),
                v5::ControlMessage::Closed(c) => Ready::Ok(c.ack()),
                v5::ControlMessage::PeerGone(c) => Ready::Ok(c.ack()),
            }))
        })
    }
}