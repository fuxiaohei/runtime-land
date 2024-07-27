use anyhow::Result;

pub(crate) fn request_ql(pid: Option<String>, uid: Option<String>, step: &str) -> Result<String> {
    if let Some(pid) = pid {
        Ok(format!(
            "sum by (typ) (increase(req_fn_total{{pid=\"{}\",typ=~\"success|error\"}}[{}]))",
            pid, step
        ))
    } else if let Some(uid) = uid {
        Ok(format!(
            "sum by (typ) (increase(req_fn_total{{uid=\"{}\",typ=~\"success|error\"}}[{}]))",
            uid, step
        ))
    } else {
        Ok(format!("sum by (typ) (increase(req_fn_total[{}]))", step))
    }
}

pub(crate) fn flow_ql(pid: Option<String>, uid: Option<String>, step: &str) -> Result<String> {
    if let Some(pid) = pid {
        Ok(format!(
            "sum by (typ) (increase(req_fn_bytes{{pid=\"{}\"}}[{}]))",
            pid, step
        ))
    } else if let Some(uid) = uid {
        Ok(format!(
            "sum by (typ) (increase(req_fn_bytes{{uid=\"{}\"}}[{}]))",
            uid, step
        ))
    } else {
        Ok(format!("sum by (typ) (increase(req_fn_bytes[{}]))", step))
    }
}

pub(crate) fn projects_traffic_ql(uid: String, pids: Vec<String>, step: &str) -> String {
    format!(
        "sum by (pid) (increase(req_fn_total{{uid=\"{}\",typ=\"all\",pid=~\"{}\"}}[{}]))",
        uid,
        pids.join("|"),
        step
    )
}

pub(crate) fn projects_flows_ql(uid: String, pids: Vec<String>, step: &str) -> String {
    format!(
        "sum by (pid,typ) (increase(req_fn_bytes{{uid=\"{}\",pid=~\"{}\"}}[{}]))",
        uid,
        pids.join("|"),
        step
    )
}
