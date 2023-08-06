import { Modal, Button, Alert } from "react-bootstrap";

function TokenRemoveModal({ show, handleClose, handleRemove, alert }) {
  return (
    <Modal show={show} onHide={handleClose}>
      <Modal.Header closeButton>
        <Modal.Title>Remove Token</Modal.Title>
      </Modal.Header>
      <Modal.Body>
        <p>Are you sure you want to remove this token?</p>
        <p>Be careful, this action cannot be undone.</p>
        {alert ? <Alert variant="danger">{alert}</Alert> : null}
      </Modal.Body>
      <Modal.Footer>
        <Button variant="secondary" onClick={handleClose}>
          Cancel
        </Button>
        <Button variant="danger" onClick={handleRemove}>
          Remove
        </Button>
      </Modal.Footer>
    </Modal>
  );
}

export default TokenRemoveModal;
