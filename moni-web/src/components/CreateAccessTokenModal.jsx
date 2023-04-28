import { Button, Modal, Form } from "react-bootstrap";

function CreateAccessTokenModal(props) {
  return (
    <Modal
      {...props}
      size="lg"
      aria-labelledby="create-access-token-modal-title"
      centered
      id="create-access-token-modal"
    >
      <Modal.Header closeButton={false}>
        <Modal.Title id="create-access-token-modal-title">
          Generate Access Token
        </Modal.Title>
      </Modal.Header>
      <Modal.Body>
        <Form>
          <Form.Group className="mb-3">
            <div className="mb-3">
              <Form.Text className="text-muted">
                Enter the description of the new access token.
              </Form.Text>
            </div>
            <Form.Control type="text" placeholder="What's the token user for" />
          </Form.Group>
        </Form>
      </Modal.Body>
      <Modal.Footer>
        <Button variant="light" onClick={props.onHide}>
          Cancel
        </Button>
        <Button variant="primary" className="ms-3">
          Create
        </Button>
      </Modal.Footer>
    </Modal>
  );
}

export default CreateAccessTokenModal;
