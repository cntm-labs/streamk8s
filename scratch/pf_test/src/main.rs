use kube::{api::Api, Client};
use k8s_openapi::api::core::v1::Pod;
use tokio::net::TcpListener;
use tokio::io::copy_bidirectional;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::try_default().await?;
    let pods: Api<Pod> = Api::namespaced(client.clone(), "kube-system");
    let pod_name = "coredns-5d78c9869d-27k88"; // Needs a valid pod name in kube-system or default
    
    // Just a dry run to see if it compiles
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Listening on 127.0.0.1:8080");

    loop {
        let (mut client_stream, _) = listener.accept().await?;
        let pods = pods.clone();
        let pod_name = pod_name.to_string();

        tokio::spawn(async move {
            let mut pf = match pods.portforward(&pod_name, &[53]).await {
                Ok(pf) => pf,
                Err(e) => {
                    eprintln!("Error starting portforward: {}", e);
                    return;
                }
            };

            let mut upstream = match pf.take_stream(53) {
                Some(stream) => stream,
                None => {
                    eprintln!("Failed to take stream for port");
                    return;
                }
            };

            if let Err(e) = copy_bidirectional(&mut client_stream, &mut upstream).await {
                eprintln!("Error copying data: {}", e);
            }
        });
    }
}
