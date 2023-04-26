import React, { useState } from "react";
import { Form, Button, Container } from "react-bootstrap";
import { Link } from "react-router-dom";
import LoginNavbar from "./LoginNavbar";
import { FaGithubAlt, FaGitlab, FaBitbucket } from "react-icons/fa";

function LoginPage() {
  return (
    <div>
      <LoginNavbar />
      <Container className="login-container">
        <h3 className="login-container-header">Login to Moni-Web</h3>
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
          <Button variant="link" href="/login-email">
            Continue with Email
          </Button>
        </div>
      </Container>
    </div>
  );
}

export default LoginPage;
