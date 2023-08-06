import { Alert, Button, Modal } from "react-bootstrap";

function ProjectRemoveModal(props) {
  let data = props.data;
  const handleRemove = async (event) => {
    event.preventDefault();
    await props.onRemove(data);
  };
  return (
    <Modal show={props.show}>
      <Modal.Header>
        <Modal.Title>Remove Token</Modal.Title>
      </Modal.Header>
      <Modal.Body>
        Are you sure you want to remove this project '
        <strong>{data.name}</strong>' ?
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
        <Button type="submit" variant="danger" size="sm" onClick={handleRemove}>
          Remove
        </Button>
      </Modal.Footer>
    </Modal>
  );
}

export default ProjectRemoveModal;
