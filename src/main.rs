use k8s_client::{
    Result,
    KubeClient,
    LogOptions
};

use stream_cancel::{Valved};
use tokio::stream::StreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    let c = KubeClient::load_conf("/home/nico/.kube/k8s-njust-kubeconfig.yaml")?;
    let pods = c.pods().await?;
    if let Some(pod) = pods.iter().find(|p|p.metadata.name.is_some() && p.metadata.name.as_ref().unwrap().starts_with("nginx")) {
        let name = pod.metadata.name.as_ref().unwrap();
        if let Ok(log_stream) = c.logs(&name, Some(
            LogOptions {
                follow: Some(true),
                since_seconds: Some(3600 * 4),
            }
        )).await {

            let (tx, rx) = tokio::sync::oneshot::channel();
            let (exit, mut inc) = Valved::new(log_stream);

            tokio::spawn(async move {
                for _ in 0..10 {
                    tokio::time::delay_for(std::time::Duration::from_millis(500)).await;
                }
                tx.send(exit).unwrap();
            });

            tokio::spawn(async move {
                while let Some(Ok(res)) = inc.next().await {
                    if let Ok(data) = String::from_utf8(res.to_vec()) {
                        print!("{}", data);
                    }
                }
            });

            rx.await.unwrap();
        }
    }

    Ok(())
}