import { Alert, Button, Modal } from "react-bootstrap";

function TokenRemoveModal(props) {
  return (
    <Modal show={props.show}>
      <Modal.Header>
        <Modal.Title>Remove Token</Modal.Title>
      </Modal.Header>
      <Modal.Body>
        Are you sure you want to remove this token '
        <strong>{props.name}</strong>' ?
        {props.alert ? (
          <Alert className="mt-4" variant="danger">
            {props.alert}
          </Alert>
        ) : null}
      </Modal.Body>
      <Modal.Footer>
        <Button variant="secondary" size="sm" onClick={props.onClose}>
          Cancel
        </Button>
        <Button
          type="submit"
          variant="danger"
          size="sm"
          onClick={props.onRemove}
        >
          Remove
        </Button>
      </Modal.Footer>
    </Modal>
  );
}

export default TokenRemoveModal;
