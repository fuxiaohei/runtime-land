import { Button, Modal, ListGroup } from "react-bootstrap";
import { TbWebhook, TbSquareKey } from "react-icons/tb";
import TimeAgo from "javascript-time-ago";

function AccessTokenRemoveModal(props) {
  const token = props.token || { updatedAt: 0, expiresAt: 0, origin: "" };
  const timeAgo = new TimeAgo("en-US");

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

      <Modal.Body>
        <p className="text-muted">
          Are you sure you want to remove this token?
        </p>
        <ListGroup className="access-tokens-list">
          <ListGroup.Item className="d-flex py-3 justify-content-between">
            <div className="desc">
              {token.origin == "dashboard" ? (
                <TbSquareKey size={20} />
              ) : (
                <TbWebhook size={20} />
              )}
              <span className="ps-1 align-text-top fw-bold">{token.name}</span>
              <span className="ps-2 extra">
                Logged {timeAgo.format(token.updatedAt * 1000)}, expires{" "}
                {timeAgo.format(token.expiresAt * 1000)}
              </span>
            </div>
          </ListGroup.Item>
        </ListGroup>
      </Modal.Body>
      <Modal.Footer>
        <Button variant="light" onClick={props.onHide}>
          Cancel
        </Button>
        <Button
          type="submit"
          variant="danger"
          className="ms-3"
          onClick={() => {
            props.onSubmit(token);
          }}
        >
          Remove
        </Button>
      </Modal.Footer>
    </Modal>
  );
}

export default AccessTokenRemoveModal;
