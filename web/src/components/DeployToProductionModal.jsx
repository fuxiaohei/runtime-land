import { Button, Modal } from "react-bootstrap";

function DeployToProductionModal(props) {
  return (
    <Modal {...props} centered id="deploy-to-production-modal">
      <Modal.Header>
        <Modal.Title>Publish this deployment to production?</Modal.Title>
      </Modal.Header>
      <Modal.Body className="lh-lg" >
        After publish, the production domain <br />
        <strong>rust-router-example.127-0-0-1.nip.io</strong>
        <br />
        will set to
        <br />
        <strong>rust-router-example-xamh7lbk.127-0-0-1.nip.io</strong>
      </Modal.Body>
      <Modal.Footer>
        <Button variant="secondary">Cancel</Button>
        <Button variant="primary">Publish</Button>
      </Modal.Footer>
    </Modal>
  );
}

export default DeployToProductionModal;
