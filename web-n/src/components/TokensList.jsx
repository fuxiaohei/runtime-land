import { ListGroup, Button } from "react-bootstrap";
import { BiTrash } from "react-icons/bi";

function TokensList() {
  return (
    <div className="access-token px-5 py-4">
      <h5 className="mb-3 fw-bold">Deployment Tokens</h5>
      <ListGroup variant="flush">
        <ListGroup.Item className="py-4">
          <div className="d-flex justify-content-between">
            <div>
              <span className="name fw-bold">Web Token</span>
              <span className="text-secondary ms-3">Expired at 2021-10-10</span>
            </div>
            <div>
              <Button className="trash-btn" variant="link">
                <BiTrash size={21} />
              </Button>
            </div>
          </div>
        </ListGroup.Item>
        <ListGroup.Item className="py-4">
          <div className="d-flex justify-content-between">
            <div>
              <span className="name fw-bold">Web Token-2</span>
            </div>
            <div>
              <Button className="trash-btn" variant="link">
                <BiTrash size={21} />
              </Button>
            </div>
          </div>
        </ListGroup.Item>
      </ListGroup>
      <div className="mt-3">
        <Button size="lg" variant="outline-secondary">
          + New Deployment Token
        </Button>
      </div>
    </div>
  );
}

export default TokensList;
