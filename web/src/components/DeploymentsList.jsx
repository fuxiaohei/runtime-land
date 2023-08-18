import { Badge, Dropdown, ListGroup } from "react-bootstrap";
import {
  BiCheckCircle,
  BiDisc,
  BiDotsVerticalRounded,
  BiErrorCircle,
} from "react-icons/bi";
import ReactTimeAgo from "react-time-ago";

function DeploymentsList({ deployments, onPublish, onDisable, onEnable }) {
  const renderRow = (deployment) => {
    let url = deployment.prod_domain
      ? deployment.prod_url
      : deployment.domain_url;

    const renderBadge = () => {
      if (deployment.prod_domain) {
        return (
          <Badge className="ms-2" bg="primary">
            Prod
          </Badge>
        );
      }
      return null;
    };

    const renderDropdown = (deployment) => {
      return (
        <Dropdown className="ps-2 d-inline-block">
          <Dropdown.Toggle as="span" variant="success">
            <BiDotsVerticalRounded />
          </Dropdown.Toggle>
          <Dropdown.Menu>
            {deployment.prod_domain ? null : (
              <Dropdown.Item onClick={() => onPublish(deployment.uuid)}>
                Publish
              </Dropdown.Item>
            )}
            {deployment.status == "active" ? (
              <Dropdown.Item onClick={() => onDisable(deployment.uuid)}>
                Disable
              </Dropdown.Item>
            ) : null}
            {deployment.status == "inactive" ? (
              <Dropdown.Item onClick={() => onEnable(deployment.uuid)}>
                Activate
              </Dropdown.Item>
            ) : null}
          </Dropdown.Menu>
        </Dropdown>
      );
    };

    const renderStatus = (deployment) => {
      if (deployment.deploy_status === "deploying") {
        return <BiDisc size={20} className="me-2 text-info" />;
      } else if (deployment.status === "inactive") {
        return <BiDisc size={20} className="me-2 text-secondary" />;
      } else if (deployment.deploy_status === "success") {
        return <BiCheckCircle size={20} className="me-2 text-success" />;
      } else if (deployment.deploy_status === "failed") {
        return <BiErrorCircle size={20} className="me-2 text-danger" />;
      } else {
        return null;
      }
    };

    return (
      <ListGroup.Item key={deployment.uuid} className="py-3">
        <div className="d-flex justify-content-between">
          <div>
            <span className="text-truncate">
              {renderStatus(deployment)}
              <a
                className="text-dark deployment-link"
                href={url}
                target="_blank"
              >
                {new URL(url).hostname}
              </a>
              {renderBadge()}
            </span>
          </div>
          <div>
            <span className="updated text-secondary">
              <ReactTimeAgo
                date={deployment.updated_at * 1000}
                locale="en-US"
              />
            </span>
            {renderDropdown(deployment)}
          </div>
        </div>
      </ListGroup.Item>
    );
  };
  return (
    <div className="mt-3 deployments-list">
      <h5 className="mb-2 fw-bold">All Deployments</h5>
      <ListGroup variant="flush">
        {deployments.map((deployment) => renderRow(deployment))}
      </ListGroup>
    </div>
  );
}

export default DeploymentsList;
