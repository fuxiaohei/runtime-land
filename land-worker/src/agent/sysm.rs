use anyhow::Result;
use metrics::gauge;
use sysinfo::{Networks, System};

/// run_sysm runs the system metrics
pub async fn run_sysm(sys: &mut System, networks: &mut Networks) -> Result<()> {
    sys.refresh_all();
    networks.refresh_list();
    networks.refresh();

    gauge!("sysm_total_memory").set(sys.total_memory() as f64);
    gauge!("sysm_used_memory").set(sys.used_memory() as f64);
    gauge!("sysm_total_swap").set(sys.total_swap() as f64);
    gauge!("sysm_used_swap").set(sys.used_swap() as f64);

    let cpu_count = sys.cpus().len() as f64;
    let mut total_cpu_usage: f64 = 0.0;
    for cpu in sys.cpus() {
        total_cpu_usage += cpu.cpu_usage() as f64;
    }
    gauge!("sysm_total_cpu_usage").set(total_cpu_usage);
    gauge!("sysm_total_cpu_usage_per_cpu").set(total_cpu_usage / cpu_count);

    for (interface_name, data) in networks.into_iter() {
        let tx_labels = vec![
            ("interface_name", interface_name.to_string()),
            ("direction", "tx".to_string()),
        ];
        gauge!("sysm_netiface", &tx_labels).set(data.total_transmitted() as f64);
        let rx_labels = vec![
            ("interface_name", interface_name.to_string()),
            ("direction", "rx".to_string()),
        ];
        gauge!("sysm_netiface", &rx_labels).set(data.total_received() as f64);
    }

    Ok(())
}
