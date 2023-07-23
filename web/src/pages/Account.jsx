import { Col, Container, Row, Form, Button, Modal } from "react-bootstrap";
import { DefaultLayout } from "../layouts/Layout";
import { useAuthContext } from "../contexts/Auth";
import { useState } from "react";

function DeploymentTokensContainer() {
  const [show, setShow] = useState(false);
  const [validated, setValidated] = useState(false);
  const [tokenDesc, setTokenDesc] = useState("");

  const handleClose = () => setShow(false);
  const handleShow = () => setShow(true);

  const handleSubmit = async (event) => {
    const form = event.currentTarget;
    const validated = form.checkValidity();
    if (validated === false) {
      event.preventDefault();
      event.stopPropagation();
      return;
    }
    setValidated(true);
    event.preventDefault();
    event.stopPropagation();
    console.log("tokenDesc", tokenDesc);
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
      <Container id="account-container">
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
