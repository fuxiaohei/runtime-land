import { Button, Modal, Alert, Form } from "react-bootstrap";
import React from "react";

function ProjectRemoveModal(props) {
  const [buttonDisabled, setButtonDisabled] = React.useState(true);
  const handleTypeName = (event) => {
    let value = event.target.value;
    if (value === props.project.name) {
      setButtonDisabled(false);
    } else {
      setButtonDisabled(true);
    }
  };

  return (
    <Modal centered show={props.show}>
      <Modal.Header>
        <Modal.Title>Delete Project</Modal.Title>
      </Modal.Header>
      <Modal.Body>
        <p>
          You are going to delete <strong>{props.project.name}</strong> project
          and all of its deployments. You will not access:
        </p>
        <p>
          <strong>{props.project.prod_url}</strong>
        </p>
        <Alert variant="danger">
          This action is not recoverable. Be careful!
        </Alert>
        <p>
          Please type <strong>{props.project.name}</strong> to continue.
        </p>
        <Form.Control
          onChange={handleTypeName}
          placeholder="project name"
          aria-label="project name"
        />
      </Modal.Body>
      <Modal.Footer>
        <Button variant="outline-secondary" onClick={props.onHide}>
          Cancel
        </Button>
        <Button
          variant={buttonDisabled ? "outline-danger" : "danger"}
          disabled={buttonDisabled}
        >
          Delete
        </Button>
      </Modal.Footer>
    </Modal>
  );
}

export default ProjectRemoveModal;
