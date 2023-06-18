import { Button, Modal, Alert } from "react-bootstrap";
import React from "react";

function DeployToProductionModal(props) {
  const [showAlert, setShowAlert] = React.useState(false);
  const [alertMessage, setAlertMessage] = React.useState("");
  const handleSubmit = async (event) => {
    event.preventDefault();
    if (props.current.id == props.prev.id) {
      setShowAlert(true);
      setAlertMessage("This deployment is already in production.");
      setTimeout(() => {
        setShowAlert(false);
      }, 3000);
      return;
    }
    await props.onSubmit();
  };
  return (
    <Modal {...props} centered id="deploy-to-production-modal">
      <Modal.Header>
        <Modal.Title>Publish this deployment to production?</Modal.Title>
      </Modal.Header>
      <Modal.Body className="lh-lg">
        After publish, the production domain <br />
        <strong>{props.produrl ? new URL(props.produrl).host : ""}</strong>
        <br />
        will proxy to
        <br />
        <strong>
          {props.current.url ? new URL(props.current.url).host : ""}
        </strong>
      </Modal.Body>
      <Modal.Footer>
        <Button variant="secondary" onClick={props.onCancel}>
          Cancel
        </Button>
        <Button
          variant="primary"
          disabled={props.loading === "true"}
          onClick={handleSubmit}
        >
          {props.loading === "true" ? "Deploying" : "Publish"}
        </Button>
        <Alert variant={"danger"} show={showAlert}>
          {alertMessage}
        </Alert>
      </Modal.Footer>
    </Modal>
  );
}

export default DeployToProductionModal;
