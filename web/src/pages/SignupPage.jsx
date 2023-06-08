import React, { useState } from "react";
import { Alert, Form, Button, Container } from "react-bootstrap";
import { Link, useNavigate } from "react-router-dom";
import LoginNavbar from "../components/LoginNavbar";
import { userAuthContext } from "../components/AuthContext";

function SignupPage() {
  const [email, setEmail] = useState("");
  const [nickname, setNickname] = useState("");
  const [password, setPassword] = useState("");
  const [validated, setValidated] = useState(false);
  const [showAlert, setShowAlert] = useState(false);
  const [alertMessage, setAlertMessage] = useState("");
  const navigate = useNavigate();
  const { signup } = userAuthContext();

  const handleSubmit = async (event) => {
    const form = event.currentTarget;
    const validated = form.checkValidity();
    if (validated === false) {
      event.preventDefault();
      event.stopPropagation();
    }
    setValidated(true);
    setShowAlert(false);
    if (validated) {
      event.preventDefault();
      event.stopPropagation();
      let res = await signup({ email, password, nickname });
      if (res.error) {
        setShowAlert(true);
        setAlertMessage(res.error);
        return;
      }
      navigate("/projects");
    }
  };

  return (
    <div>
      <LoginNavbar />
      <Container className="login-container">
        <h3 className="login-container-header">Sign up Moni-Web</h3>
        <hr />
        <Form
          id="login-email-form"
          noValidate
          validated={validated}
          onSubmit={handleSubmit}
        >
          <Form.Group className="mb-3" controlId="formBasicEmail">
            <Form.Label>Email address</Form.Label>
            <Form.Control
              type="email"
              placeholder="Enter email"
              required
              value={email}
              onChange={(e) => setEmail(e.target.value)}
            />
            <Form.Control.Feedback type="invalid">
              Please provide a valid email.
            </Form.Control.Feedback>
          </Form.Group>

          <Form.Group className="mb-3" controlId="formDisplayName">
            <Form.Label>Display Name</Form.Label>
            <Form.Control
              type="text"
              placeholder="Enter display name or nickname"
              required
              value={nickname}
              onChange={(e) => setNickname(e.target.value)}
            />
            <Form.Control.Feedback type="invalid">
              Please provide a nickname.
            </Form.Control.Feedback>
          </Form.Group>

          <Form.Group className="mb-4" controlId="formBasicPassword">
            <Form.Label>Password</Form.Label>
            <Form.Control
              type="password"
              placeholder="Password"
              required
              value={password}
              onChange={(e) => setPassword(e.target.value)}
            />
            <Form.Control.Feedback type="invalid">
              Please input a password.
            </Form.Control.Feedback>
          </Form.Group>
          <Alert variant={"danger"} show={showAlert}>
            {alertMessage}
          </Alert>
          <div className="d-flex mb-4 justify-content-between">
            <Button variant="primary" type="submit" className="w-100">
              Sign up
            </Button>
          </div>
          <div className="d-flex justify-content-between login-email-link">
            <Link to="/login-email">Have an account? Sign In</Link>
          </div>
        </Form>
      </Container>
    </div>
  );
}

export default SignupPage;
