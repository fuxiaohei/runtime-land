import React from "react";
import { Button, Container } from "react-bootstrap";
import LoginNavbar from "../components/LoginNavbar";
import { FaGithubAlt, FaGitlab, FaBitbucket } from "react-icons/fa";
import { ButtonLink } from "../components/ButtonLink";

function LoginPage() {
  return (
    <div>
      <LoginNavbar />
      <Container className="login-container">
        <h3 className="login-container-header">Login to Runtime.land</h3>
        <hr />
        <div className="d-grid gap-2 login-connection">
          <Button className="item" variant="dark" disabled>
            <FaGithubAlt size="24" className="fa" /> Continue with Github
          </Button>
          <Button className="item" variant="danger" disabled>
            <FaGitlab size="24" className="fa" /> Continue with Gitlab
          </Button>
          <Button variant="primary" disabled>
            <FaBitbucket size="24" className="fa" /> Continue with Bitbucket
          </Button>
        </div>
        <hr />
        <div className="d-grid gap-2">
          <ButtonLink variant="link" to="/login-email">
            Continue with Email
          </ButtonLink>
        </div>
      </Container>
    </div>
  );
}

export default LoginPage;
