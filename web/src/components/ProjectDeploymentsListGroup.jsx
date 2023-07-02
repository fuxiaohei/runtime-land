import { ListGroup, Dropdown, Button, Modal, Alert } from "react-bootstrap";
import { BsCheck2Circle, BsAppIndicator } from "react-icons/bs";
import TimeAgo from "javascript-time-ago";
import React from "react";

function ProjectDeploymentRemoveModal(props) {
  const [loading, setLoading] = React.useState(false);
  const handleRemoveCilck = async (event) => {
    event.preventDefault();
    setLoading(true);
    await props.onRemove();
    setLoading(false);
  };
  return (
    <Modal centered show={props.show}>
      <Modal.Header>
        <Modal.Title>Remove Deployment</Modal.Title>
      </Modal.Header>
      <Modal.Body>
        <p>You are going to remove this deployment. You will not access:</p>
        <p>
          <strong>{props.deployment.domain_url}</strong>
        </p>
        <Alert variant="danger">
          This action is not recoverable. Be careful!
        </Alert>
      </Modal.Body>
      <Modal.Footer>
        <Button variant="outline-secondary" onClick={props.onHide}>
          Cancel
        </Button>
        <Button
          variant={"danger"}
          disabled={loading}
          onClick={handleRemoveCilck}
        >
          {loading ? "Removing" : "Remove"}
        </Button>
      </Modal.Footer>
    </Modal>
  );
}

function ProjectDeploymentsListGroup({
  deploymentsList,
  onDeployToProduction,
  onRemoveDeployment,
}) {
  const timeAgo = new TimeAgo("en-US");
  const buildHandleDeployToProduction = (deployment) => {
    return async (event) => {
      onDeployToProduction(deployment);
    };
  };
  const [modalShow, setModalShow] = React.useState(false);
  const [currentRemoveDeployment, setCurrentRemoveDeployment] = React.useState(
    {}
  );

  const handleHideModal = () => {
    setModalShow(false);
  };

  const renderDeployToProductionButton = (deployment) => {
    if (deployment.prod_status === 1) {
      return (
        <Dropdown.Item className="small" disabled>
          In Production
        </Dropdown.Item>
      );
    }
    return (
      <Dropdown.Item
        className="small"
        onClick={buildHandleDeployToProduction(deployment)}
      >
        Deploy to Production
      </Dropdown.Item>
    );
  };

  const renderRemoveButton = (deployment) => {
    // if deployment is in production, don't show remove button
    if (deployment.prod_status === 1) {
      return null;
    }
    return (
      <Dropdown.Item
        className="small text-danger-emphasis"
        onClick={() => {
          setModalShow(true);
          setCurrentRemoveDeployment(deployment);
        }}
      >
        Remove
      </Dropdown.Item>
    );
  };

  const handleDeploymentRemove = async () => {
    console.log("remove deployment", currentRemoveDeployment);
    await onRemoveDeployment(currentRemoveDeployment);
    setModalShow(false);
  };

  const listItems = deploymentsList.map((deployment) => (
    <ListGroup.Item
      className="lh-lg d-flex justify-content-between ps-0"
      key={deployment.uuid}
    >
      <div className="deployment-metadata text-truncate">
        <BsCheck2Circle className="status-icon me-2" size={20} />
        <a className="name" href={deployment.domain_url} target="_blank">
          {new URL(deployment.domain_url).host}
        </a>
      </div>
      <div className="deployment-promotion">
        <span className="time-ago small text-muted">
          {timeAgo.format(deployment.updated_at * 1000)}
        </span>
        <Dropdown className="promote-btn ms-2 d-inline-block">
          <Dropdown.Toggle as="a" className="cursor-pointer">
            <BsAppIndicator size={12} />
          </Dropdown.Toggle>
          <Dropdown.Menu className="lh-1 text-muted">
            {renderDeployToProductionButton(deployment)}
            <Dropdown.Divider />
            <Dropdown.Item className="small">Logs</Dropdown.Item>
            {deployment.prod_status !== 1 ? <Dropdown.Divider /> : null}
            {renderRemoveButton(deployment)}
          </Dropdown.Menu>
        </Dropdown>
      </div>
    </ListGroup.Item>
  ));

  return (
    <div>
      <ListGroup variant="flush" className="project-deployment-list">
        {listItems}
      </ListGroup>
      <ProjectDeploymentRemoveModal
        show={modalShow}
        onHide={handleHideModal}
        deployment={currentRemoveDeployment}
        onRemove={handleDeploymentRemove}
      />
    </div>
  );
}

export default ProjectDeploymentsListGroup;
