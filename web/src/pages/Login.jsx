import { LoginLayout, LoginSidebar } from "../layouts/Login";
import {
  Button,
  Row,
  Col,
  Container,
  Form,
  FloatingLabel,
} from "react-bootstrap";
import { useState } from "react";
import { Link } from "react-router-dom";

function LoginPage() {
  const [validated, setValidated] = useState(false);

  const handleSubmit = (event) => {
    const form = event.currentTarget;
    if (form.checkValidity() === false) {
      event.preventDefault();
      event.stopPropagation();
    }

    setValidated(true);
  };
  return (
    <LoginLayout>
      <Container id="login-container">
        <Row>
          <Col className="text-center">
            <LoginSidebar />
          </Col>
          <Col className="text-start login-right-side pb-3 ps-5 border-start">
            <Form noValidate validated={validated} onSubmit={handleSubmit}>
              <FloatingLabel
                controlId="floatingInput"
                label="Email address"
                className="mb-3"
              >
                <Form.Control
                  type="email"
                  required
                  name="email"
                  placeholder="name@example.com"
                />
                <Form.Control.Feedback type="invalid">
                  Please enter a valid email address.
                </Form.Control.Feedback>
              </FloatingLabel>
              <FloatingLabel
                controlId="floatingPassword"
                label="Password"
                className="mb-3"
              >
                <Form.Control
                  name="password"
                  type="password"
                  required
                  placeholder="Password"
                />
                <Form.Control.Feedback type="invalid">
                  Please enter a valid password.
                </Form.Control.Feedback>
              </FloatingLabel>
              <Button variant="primary" type="submit">
                Login
              </Button>
            </Form>
            <hr />
            <div className="d-flex justify-content-between">
              <div>
                <Link to="/signup">Sign up</Link>
              </div>
              <div>
                <Link to="/forget-password">Forget Password</Link>
              </div>
            </div>
          </Col>
        </Row>
      </Container>
    </LoginLayout>
  );
}

export default LoginPage;
