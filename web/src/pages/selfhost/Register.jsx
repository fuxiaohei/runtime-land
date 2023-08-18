import { useMutation } from "@tanstack/react-query";
import { useState } from "react";
import { Alert, Button, Container, FloatingLabel, Form } from "react-bootstrap";
import { Helmet } from "react-helmet-async";
import { Link } from "react-router-dom";
import { handleTokenResponse } from "../../api/client";
import { signup } from "../../api/sign";

function RegisterPage() {
  const mutation = useMutation(
    async (data) => {
      return await signup(data);
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
        <title>Sign up | Runtime.land</title>
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
        <h3 className="mb-4">Sign up Runtime.land</h3>
        <FloatingLabel label="Email address" className="mb-4">
          <Form.Control
            size="lg"
            type="email"
            name="email"
            placeholder="name@example.com"
            required
          />
        </FloatingLabel>
        <FloatingLabel label="Display name" className="mb-4">
          <Form.Control
            size="lg"
            type="text"
            name="nickname"
            placeholder="your name"
            required
          />
        </FloatingLabel>
        <FloatingLabel label="Password" className="mb-4">
          <Form.Control size="lg" type="password" name="password" required />
        </FloatingLabel>
        {mutation.isError ? (
          <Alert variant="danger">{mutation.error.toString()}</Alert>
        ) : null}
        <hr className="mb-4" />
        <div className="text-end mb-4">
          <Button
            variant="primary"
            disabled={mutation.isLoading}
            type="submit"
            form="login-form"
          >
            {mutation.isLoading ? "Loading..." : "Create new account"}
          </Button>
        </div>
        <div className="text-center lh-lg">
          <span className="me-2">Already have an account?</span>
          <Link to="/login" className="text-dark">
            Sign in
          </Link>
        </div>
      </Form>
    </Container>
  );
}

export default RegisterPage;
