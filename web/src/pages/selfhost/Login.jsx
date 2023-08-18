import { useMutation } from "@tanstack/react-query";
import { useState } from "react";
import { Alert, Button, Container, FloatingLabel, Form } from "react-bootstrap";
import { Helmet } from "react-helmet-async";
import { Link } from "react-router-dom";
import { handleTokenResponse } from "../../api/client";
import { login_by_email } from "../../api/sign";

function LoginPage() {
  const mutation = useMutation(
    async (data) => {
      return await login_by_email(data);
    },
    {
      onSuccess: (data) => {
        handleTokenResponse(data);
        // reload page to get new token
        window.location.href = "/projects";
      },
    }
  );

  const [validated, setValidated] = useState(false);

  const handleSubmit = async (event) => {
    const form = event.currentTarget;
    setValidated(true);
    if (form.checkValidity() === false) {
      event.preventDefault();
      event.stopPropagation();
      return;
    }

    event.preventDefault();
    const formData = new FormData(form);
    const values = Object.fromEntries(formData.entries());
    mutation.mutate(values);
    setValidated(false);
  };

  return (
    <Container className="text-center mt-5">
      <Helmet>
        <title>Login | Runtime.land</title>
      </Helmet>
      <div className="mb-4">
        <img width={80} src="/public/logo-v2.svg" />
      </div>
      <Form
        id="login-form"
        className="mx-auto"
        noValidate
        validated={validated}
        onSubmit={handleSubmit}
      >
        <h3 className="mb-4">Login Runtime.land</h3>
        <FloatingLabel label="Email address" className="mb-4">
          <Form.Control
            size="lg"
            type="email"
            name="email"
            placeholder="name@example.com"
            required
          />
        </FloatingLabel>
        <FloatingLabel label="Password" className="mb-4">
          <Form.Control size="lg" type="password" name="password" required />
        </FloatingLabel>
        <div className="text-end lh-lg mb-4">
          <Link to="/forgot-password" className="text-dark">
            Forgot password?
          </Link>
        </div>
        {mutation.isError ? (
          <Alert variant="danger">{mutation.error.toString()}</Alert>
        ) : null}
        <hr className="mb-4" />
        <div className="text-end mb-4">
          <Button
            disabled={mutation.isLoading}
            variant="primary"
            type="submit"
            form="login-form"
          >
            {mutation.isLoading ? "Loading..." : "Login"}
          </Button>
        </div>
        <div className="text-center lh-lg">
          <span className="me-2">Don't have an account?</span>
          <Link to="/register" className="text-dark">
            Register
          </Link>
        </div>
      </Form>
    </Container>
  );
}

export default LoginPage;
