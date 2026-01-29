#[macro_export]
macro_rules! http {
    (hostname => &$hostname:ident, port => &$port:ident, interface => &$interface:ident) => {
        use vetis::{
            config::{ListenerConfig, ServerConfig, VirtualHostConfig},
            errors::VetisError,
            server::virtual_host::{DefaultVirtualHost, VirtualHost},
            Vetis,
        };

        let listener = ListenerConfig::builder()
            .hostname($hostname)
            .port($port)
            .interface($interface)
            .build();

        let config = ServerConfig::builder()
            .add_listener(listener)
            .build();

        let virtual_host_config = VirtualHostConfig::builder()
            .hostname($hostname)
            .port($port)
            .build();

        let virtual_host = DefaultVirtualHost::new(virtual_host_config);

        let mut vetis = vetis::Vetis::new(config);

        vetis
            .add_virtual_host(virtual_host)
            .await;

        Ok(vetis)
    };

    (hostname => $hostname:literal, port => $port:literal, interface => $interface:literal, handler => $handler:ident) => {
        async move {
            use vetis::{
                config::{ListenerConfig, ServerConfig, VirtualHostConfig},
                errors::VetisError,
                server::virtual_host::{DefaultVirtualHost, VirtualHost},
                Vetis,
            };

            let listener = ListenerConfig::builder()
                .port($port)
                .interface($interface.to_string())
                .build();

            let config = ServerConfig::builder()
                .add_listener(listener)
                .build();

            let virtual_host_config = VirtualHostConfig::builder()
                .hostname($hostname.to_string())
                .port($port)
                .build()?;

            let mut virtual_host = DefaultVirtualHost::new(virtual_host_config);
            virtual_host.set_handler($handler);

            let mut vetis = Vetis::new(config);

            vetis
                .add_virtual_host(virtual_host)
                .await;

            Ok::<Vetis, Box<VetisError>>(vetis)
        }
    };
}
