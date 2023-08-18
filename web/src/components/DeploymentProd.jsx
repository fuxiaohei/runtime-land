import { BiLinkExternal, BiTime } from "react-icons/bi";
import ReactTimeAgo from "react-time-ago";

function DeploymentProd({ project }) {
  if (!project.prod_url) {
    return (
      <div className="border-bottom border-top deployment-production">
        <h5 className="my-3 text-secondary fw-bold">No Production Domains</h5>
      </div>
    );
  }

  return (
    <div className="border-bottom border-top deployment-production">
      <h5 className="my-3 fw-bold">Production Domains</h5>
      <div className="prodution ms-4">
        <div className="d-flex justify-content-between">
          <div>
            <p className="fs-6">
              <BiLinkExternal className="me-1" />
              <a
                href={project.prod_url}
                target="_blank"
                className="text-dark deployment-link fw-bold"
              >
                {new URL(project.prod_url).host}
              </a>
            </p>
            <p>
              <BiLinkExternal className="me-1" />
              <a
                href={project.deployment_url}
                target="_blank"
                className="text-secondary deployment-link"
              >
                {new URL(project.deployment_url).host}
              </a>
            </p>
          </div>
        </div>
        <p className="text-secondary">
          <BiTime className="me-1" />
          deployed at{" "}
          <ReactTimeAgo date={project.updated_at * 1000} locale="en-US" />
        </p>
      </div>
    </div>
  );
}

export default DeploymentProd;
