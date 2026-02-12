#[macro_export]
macro_rules! http {
    (hostname => $hostname:expr, root_directory => $root_directory:expr, port => $port:expr, interface => $interface:expr, handler => $handler:ident) => {
        async move {
            use vetis::{
                config::server::{virtual_host::VirtualHostConfig, ListenerConfig, ServerConfig},
                errors::VetisError,
                server::virtual_host::{path::HandlerPath, VirtualHost},
                Vetis,
            };

            let listener = ListenerConfig::builder()
                .port($port)
                .interface($interface)
                .build()
                .expect("Failed to configure listener");

            let config = ServerConfig::builder()
                .add_listener(listener)
                .build()
                .expect("Failed to configure server");

            let virtual_host_config = VirtualHostConfig::builder()
                .hostname($hostname)
                .root_directory($root_directory)
                .port($port)
                .build()?;

            let mut virtual_host = VirtualHost::new(virtual_host_config);

            let root_path = HandlerPath::builder()
                .uri("/")
                .handler(Box::new($handler))
                .build()
                .unwrap();

            virtual_host.add_path(root_path);

            let mut vetis = vetis::Vetis::new(config);

            vetis
                .add_virtual_host(virtual_host)
                .await;

            Ok::<Vetis, Box<VetisError>>(vetis)
        }
    };

    (hostname => $hostname:literal, port => $port:literal, interface => $interface:literal, handler => $handler:ident) => {
        async move {
            use vetis::{
                config::server::{virtual_host::VirtualHostConfig, ListenerConfig, ServerConfig},
                errors::VetisError,
                server::{path::HandlerPath, virtual_host::VirtualHost},
                Vetis,
            };

            let listener = ListenerConfig::builder()
                .port($port)
                .interface($interface)
                .build();

            let config = ServerConfig::builder()
                .add_listener(listener)
                .build();

            let virtual_host_config = VirtualHostConfig::builder()
                .hostname($hostname)
                .port($port)
                .build()?;

            let mut virtual_host = VirtualHost::new(virtual_host_config);

            let root_path = HandlerPath::builder()
                .uri("/")
                .handler(Box::new($handler))
                .build()
                .unwrap();

            virtual_host.add_path(root_path);

            let mut vetis = Vetis::new(config);

            vetis
                .add_virtual_host(virtual_host)
                .await;

            Ok::<Vetis, Box<VetisError>>(vetis)
        }
    };
}

#[macro_export]
macro_rules! https {
    (hostname => &$hostname:ident, port => &$port:ident, interface => &$interface:ident, &cert => &$cert:ident, &key => &$key:ident) => {
        use vetis::{
            config::{ListenerConfig, ServerConfig, VirtualHostConfig},
            errors::VetisError,
            server::{path::HandlerPath, virtual_host::VirtualHost},
            Vetis,
        };

        let listener = ListenerConfig::builder()
            .port($port)
            .interface($interface)
            .build();

        let config = ServerConfig::builder()
            .add_listener(listener)
            .build();

        let security_config = SecurityConfig::builder()
            .cert_from_file($cert)
            .key_from_file($key)
            .build();

        let virtual_host_config = VirtualHostConfig::builder()
            .hostname($hostname)
            .port($port)
            .security(security_config)
            .build()?;

        let mut virtual_host = VirtualHost::new(virtual_host_config);

        let root_path = HandlerPath::builder()
            .uri("/")
            .handler(Box::new($handler))
            .build()
            .unwrap();

        virtual_host.add_path(root_path);

        let mut vetis = Vetis::new(config);

        vetis
            .add_virtual_host(virtual_host)
            .await;

        Ok::<Vetis, Box<VetisError>>(vetis)
    };
}
