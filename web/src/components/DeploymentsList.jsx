import { Badge, Dropdown, ListGroup } from "react-bootstrap";
import { BiDotsVerticalRounded, BiCheckCircle, BiDisc } from "react-icons/bi";
import ReactTimeAgo from "react-time-ago";

function DeploymentsList({ deployments }) {
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

    const renderDropdown = () => {
      return (
        <Dropdown className="ps-2 d-inline-block">
          <Dropdown.Toggle as="span" variant="success">
            <BiDotsVerticalRounded />
          </Dropdown.Toggle>
          <Dropdown.Menu>
            {deployment.prod_domain ? null : (
              <Dropdown.Item href="#/action-1">Publish</Dropdown.Item>
            )}
            <Dropdown.Item href="#/action-2">Disable</Dropdown.Item>
            {deployment.prod_domain ? null : (
              <Dropdown.Item className="text-danger" href="#/action-3">
                Remove
              </Dropdown.Item>
            )}
          </Dropdown.Menu>
        </Dropdown>
      );
    };

    const renderStatus = () => {
      if (deployment.deploy_status === "success") {
        return <BiCheckCircle className="me-2 text-success" />;
      } else if (deployment.deploy_status === "deploying") {
        return <BiDisc className="me-2 text-info" />;
      } else {
        return null;
      }
    };

    return (
      <ListGroup.Item key={deployment.uuid} className="py-3">
        <div className="d-flex justify-content-between">
          <div>
            <span className="text-truncate">
              {renderStatus()}
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
            {renderDropdown()}
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
