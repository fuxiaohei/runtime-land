import { ListGroup, Dropdown } from "react-bootstrap";
import { BsCheck2Circle, BsAppIndicator } from "react-icons/bs";
import TimeAgo from "javascript-time-ago";

function ProjectDeploymentsListGroup({ deploymentsList }) {
  const timeAgo = new TimeAgo("en-US");
  const listItems = deploymentsList.map((deployment) => (
    <ListGroup.Item
      className="lh-lg d-flex justify-content-between ps-0"
      key={deployment.uuid}
    >
      <div className="deployment-metadata text-truncate">
        <BsCheck2Circle className="status-icon me-2" size={20} />
        <a className="name" href={deployment.url} target="_blank">
          {deployment.domain}.127-0-0-1.nip.io
        </a>
      </div>
      <div className="deployment-promotion">
        <span className="time-ago small text-muted">
          {timeAgo.format(deployment.updatedAt * 1000)}
        </span>
        <Dropdown className="promote-btn ms-2 d-inline-block">
          <Dropdown.Toggle as="a" className="cursor-pointer">
            <BsAppIndicator size={12} />
          </Dropdown.Toggle>
          <Dropdown.Menu className="lh-1 text-muted">
            <Dropdown.Item className="small">
              Deploy to Production
            </Dropdown.Item>
            <Dropdown.Divider />
            <Dropdown.Item className="small">Logs</Dropdown.Item>
          </Dropdown.Menu>
        </Dropdown>
      </div>
    </ListGroup.Item>
  ));

  return (
    <ListGroup variant="flush" className="project-deployment-list">
      {listItems}
    </ListGroup>
  );
}

export default ProjectDeploymentsListGroup;
