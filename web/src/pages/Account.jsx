import {
  Col,
  Container,
  Row,
  Form,
  Button,
  Modal,
  ListGroup,
  Spinner,
} from "react-bootstrap";
import { DefaultLayout } from "../layouts/Layout";
import { useAuthContext } from "../contexts/Auth";
import { useEffect, useState } from "react";
import { createDeploymentToken, listDeploymentTokens } from "../api/token";
import { FaRegCopy, FaCheck } from "react-icons/fa";
import { RiDeleteBinLine } from "react-icons/ri";
import { CopyToClipboard } from "react-copy-to-clipboard";
import { DateTime } from "luxon";
import { useToastContext } from "../contexts/Toast";

function DeploymentTokensContainer() {
  const [show, setShow] = useState(false);
  const [validated, setValidated] = useState(false);
  const [tokenDesc, setTokenDesc] = useState("");
  const [newToken, setNewToken] = useState(null);
  const [copied, setCopied] = useState(false);
  const { toastError } = useToastContext();

  const handleClose = () => setShow(false);
  const handleShow = () => setShow(true);

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

    let response = await createDeploymentToken(tokenDesc);
    if (response.error) {
      setShow(false);
      toastError(response.error);
      return;
    }
    setNewToken(response);
    setShow(false);
  };

  const renderNewToken = () => {
    if (!newToken) {
      return null;
    }
    return (
      <div className="new-token border p-3 rounded d-flex justify-content-between mb-4">
        <div>
          <p className="mb-1">
            <strong>{newToken.name}</strong>
          </p>
          <p className="mb-1">
            {newToken.value}
            <CopyToClipboard
              role="button"
              text={newToken.value}
              onCopy={() => setCopied(true)}
            >
              {copied ? (
                <FaCheck className="ms-2" />
              ) : (
                <FaRegCopy className="ms-2" />
              )}
            </CopyToClipboard>
          </p>
        </div>
        <Button
          variant="success"
          onClick={() => {
            setNewToken(null);
            fetchTokens();
          }}
        >
          Done
        </Button>
      </div>
    );
  };

  const [isLoading, setIsLoading] = useState(true);
  const [tokens, setTokens] = useState([1, 2, 3]);

  const fetchTokens = async () => {
    setIsLoading(true);
    let response = await listDeploymentTokens();
    if (response.error) {
      console.log(response.error);
      return;
    }
    setTokens(response);
    setIsLoading(false);
  };

  useEffect(() => {
    fetchTokens();
  }, []);

  const renderTokens = () => {
    if (isLoading) {
      return (
        <Spinner
          className="my-3"
          variant="secondary"
          animation="border"
          role="status"
        >
          <span className="visually-hidden">Loading...</span>
        </Spinner>
      );
    }
    return tokens.map((token) => {
      return (
        <ListGroup.Item
          key={token.uuid}
          className="d-flex justify-content-between lh-lg"
        >
          <div>
            <strong>{token.name}</strong>
            <span className="ms-2 text-secondary">
              expires in{" "}
              {DateTime.fromSeconds(token.expired_at)
                .setLocale("en-US")
                .toLocaleString(DateTime.DATE_FULL)}
            </span>
          </div>
          <div>
            <RiDeleteBinLine size={21} role="button" />
          </div>
        </ListGroup.Item>
      );
    });
  };

  return (
    <Col md={8}>
      <div className="mt-3 d-flex justify-content-between">
        <h4>Deployment Tokens</h4>
        <Button variant="primary" size="sm" onClick={handleShow}>
          Create New Token
        </Button>
      </div>
      <hr />
      <p className="text-secondary">
        Deployment tokens are used to authenticate with the api when deploying a
        project via the command line.
      </p>
      <pre className="text-secondary">
        land-cli deploy --token=&lt;token&gt;
      </pre>
      {renderNewToken()}
      <ListGroup className="tokens-list">{renderTokens()}</ListGroup>
      <Modal show={show}>
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
          </Modal.Body>
          <Modal.Footer>
            <Button variant="secondary" size="sm" onClick={handleClose}>
              Cancel
            </Button>
            <Button type="submit" variant="primary" size="sm">
              Create
            </Button>
          </Modal.Footer>
        </Form>
      </Modal>
    </Col>
  );
}

function AccountPage() {
  const { user } = useAuthContext();
  return (
    <DefaultLayout title="Account | Runtime.land">
      <Container fluid id="account-container">
        <Row>
          <Col md={4}>
            <h4 className="mt-3 account-info-title">Account Infomation</h4>
            <hr />
            <Form>
              <Form.Group className="mb-3">
                <Form.Label>Name</Form.Label>
                <Form.Control type="text" disabled defaultValue={user.name} />
              </Form.Group>
              <Form.Group className="mb-3">
                <Form.Label>Email</Form.Label>
                <Form.Control type="email" disabled defaultValue={user.email} />
              </Form.Group>
            </Form>
          </Col>
          <DeploymentTokensContainer />
        </Row>
      </Container>
    </DefaultLayout>
  );
}

export default AccountPage;
