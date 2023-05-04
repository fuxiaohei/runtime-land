import { Button, Modal, Form } from "react-bootstrap";

function AccessTokenRemoveModal(props) {
  return (
    <Modal
      {...props}
      size="lg"
      aria-labelledby="remove-access-token-modal-title"
      centered
      id="remove-access-token-modal"
    >
      <Modal.Header closeButton={false}>
        <Modal.Title id="Remove token-access-token-modal-title">
          Remove Access Token
        </Modal.Title>
      </Modal.Header>
      <Form onSubmit={props.onSubmit}>
        <Modal.Body>
          <Form.Group className="mb-3">
            <div className="mb-3">
              <Form.Text className="text-muted">
                Are you sure you want to remove this token?
              </Form.Text>
            </div>
            <Form.Control
              name="tokenvalue"
              required
              type="text"
              placeholder="What's the token user for"
            />
          </Form.Group>
        </Modal.Body>
        <Modal.Footer>
          <Button variant="light" onClick={props.onHide}>
            Cancel
          </Button>
          <Button type="submit" variant="danger" className="ms-3">
            Remove
          </Button>
        </Modal.Footer>
      </Form>
    </Modal>
  );
}

export default AccessTokenRemoveModal;
