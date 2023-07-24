import { useState } from "react";
import { Form, Button, Modal, Alert } from "react-bootstrap";

function TokenCreateModal(props) {
  const [validated, setValidated] = useState(false);
  const [tokenDesc, setTokenDesc] = useState("");

  const handleSubmit = async (event) => {
    const form = event.currentTarget;
    const validated = form.checkValidity();
    if (validated === false) {
      event.preventDefault();
      setValidated(true);
      return;
    }
    setValidated(true);
    event.preventDefault();

    await props.onCreate(tokenDesc);
  };

  return (
    <Modal show={props.show}>
      <Modal.Header>
        <Modal.Title>Create new token</Modal.Title>
      </Modal.Header>
      <Form noValidate validated={validated} onSubmit={handleSubmit}>
        <Modal.Body>
          <Form.Group className="mb-3">
            <div className="mb-3">
              <Form.Text className="text-muted">
                Enter the description of the new deployment token.
              </Form.Text>
            </div>
            <Form.Control
              name="tokenvalue"
              required
              type="text"
              placeholder="What's the token user for"
              value={tokenDesc}
              onChange={(e) => setTokenDesc(e.target.value)}
            />
            <Form.Control.Feedback type="invalid">
              Please enter a valid token description.
            </Form.Control.Feedback>
          </Form.Group>
          {props.alert ? (
            <Alert dismissible variant="danger">
              {props.alert}
            </Alert>
          ) : null}
        </Modal.Body>
        <Modal.Footer>
          <Button variant="secondary" size="sm" onClick={props.onClose}>
            Cancel
          </Button>
          <Button type="submit" variant="primary" size="sm">
            Create
          </Button>
        </Modal.Footer>
      </Form>
    </Modal>
  );
}

export default TokenCreateModal;
