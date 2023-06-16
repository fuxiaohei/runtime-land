import { Link } from "react-router-dom";
import { ButtonLink } from "../components/ButtonLink";
import DashboardNavbar from "../components/DashboardNavbar";
import {
  Container,
  Button,
  Row,
  Col,
  Form,
  InputGroup,
  Card,
  Spinner,
} from "react-bootstrap";
import React, { useEffect } from "react";
import ProjectsGroup from "../components/ProjectsGroup";
import { listProjects } from "../api/project";

function ProjectsPage() {
  const [loadingStatus, setLoadingStatus] = React.useState({
    loading: true,
  });
  const [projects, setProjects] = React.useState({
    data: [],
    counter: 0,
  });

  const fetchProjects = async () => {
    let response = await listProjects();
    if (response.error) {
      return;
    }
    setLoadingStatus({ loading: false });
    setProjects({
      data: response.dataList || [],
      counter: response.dataList.length,
    });
  };

  useEffect(() => {
    if (loadingStatus.loading) {
      fetchProjects();
    }
  });

  return (
    <div>
      <DashboardNavbar />
      <Container id="dashboard-container">
        <header id="dashboard-header">
          <Container>
            <Row>
              <Col md={4} id="dashboard-header-left">
                <h2>Projects</h2>
                <h3>
                  {loadingStatus.loading ? (
                    <span>Loading</span>
                  ) : (
                    <strong>{projects.counter}</strong>
                  )}{" "}
                  Projects
                </h3>
              </Col>
              <Col id="dashboard-header-right">
                <ButtonLink to="/new" variant="primary">
                  + New Project
                </ButtonLink>
              </Col>
            </Row>
          </Container>
        </header>
        <section id="dashboard-projects" className="pt-2">
          {loadingStatus.loading ? (
            <Spinner className="projects-loading m-4" animation="border" />
          ) : (
            <ProjectsGroup projects={projects} />
          )}
        </section>
      </Container>
    </div>
  );
}

export default ProjectsPage;
