import {
  Tab,
  Col,
  Row,
  Nav,
  Container,
  Spinner,
  Card,
  Form,
  InputGroup,
  Button,
} from "react-bootstrap";
import DashboardNavbar from "../components/DashboardNavbar";
import ProjectHeader from "../components/ProjectHeader";
import ProjectTabs from "../components/ProjectTabs";
import { useParams } from "react-router-dom";
import { Helmet } from "react-helmet-async";
import React, { useEffect } from "react";
import { getProjectOverview } from "../cloud/projects";
import ProjectRemoveModal from "../components/ProjectRemoveModal";

function ProjectSettingsPage() {
  const { projectName } = useParams();
  const [loadingStatus, setLoadingStatus] = React.useState(true);
  const [projectOverview, setProjectOverview] = React.useState(null);
  const [removeModelShow, setRemoveModelShow] = React.useState(false);

  const handleRemoveClick = async () => {
    setRemoveModelShow(true);
  };

  const fetchProjectOverview = async () => {
    let project = await getProjectOverview(projectName, false);
    if (project.error) {
      return;
    }
    setProjectOverview(project);
    setLoadingStatus(false);
  };

  useEffect(() => {
    if (!projectOverview) {
      fetchProjectOverview();
    }
  });

  if (loadingStatus) {
    return (
      <div>
        <Helmet>
          <title>{projectName} | Project | Runtime.land</title>
        </Helmet>
        <DashboardNavbar />
        <Container id="project-container">
          <ProjectHeader
            projectName={projectName}
            project={projectOverview || {}}
          />
          <Container>
            <Spinner className="m-4" animation="border" />
          </Container>
        </Container>
      </div>
    );
  }

  return (
    <div>
      <Helmet>
        <title>{projectName} | Project | Runtime.land</title>
      </Helmet>
      <DashboardNavbar />
      <Container id="project-container">
        <ProjectHeader
          projectName={projectName}
          project={projectOverview || {}}
        />
        <ProjectTabs projectName={projectName} activeKey="settings" />
        <div id="project-overview-container" className="pt-4 pb-5">
          <Container className="project-settings-container">
            <Tab.Container defaultActiveKey="project-name">
              <Row id="project-settings-tabs">
                <Col sm={8}>
                  <Tab.Content>
                    <Tab.Pane eventKey="project-name">
                      <Card className="project-name-card">
                        <Card.Body>
                          <Card.Title>Project Name</Card.Title>
                          <Card.Text as="div">
                            <p className="text-secondary fs-6">
                              The name of your project. If project name is
                              changed, the project URL will also be changed.
                            </p>
                            <InputGroup className="mb-3">
                              <Form.Control
                                placeholder="project name"
                                aria-label="project name"
                                aria-describedby="project-domain-suffix"
                                defaultValue={projectOverview.name}
                              />
                              <InputGroup.Text
                                className="bg-primary-subtle"
                                id="project-domain-suffix"
                              >
                                {window.PROD_DOMAIN}
                              </InputGroup.Text>
                            </InputGroup>
                            <Button variant="outline-secondary" disabled>
                              Save
                            </Button>
                          </Card.Text>
                        </Card.Body>
                      </Card>
                    </Tab.Pane>
                    <Tab.Pane eventKey="danger-zone">
                      <Card className="project-danger-zone">
                        <Card.Body>
                          <Card.Title>Danger Zone</Card.Title>
                          <Card.Text as="div">
                            <p className="text-secondary fs-6">
                              Delete <strong>{projectOverview.name}</strong> and
                              all of its deployments. This action is not
                              recoverable. Be careful!
                            </p>
                            <Button
                              variant="outline-danger"
                              onClick={handleRemoveClick}
                            >
                              Delete
                            </Button>
                          </Card.Text>
                        </Card.Body>
                      </Card>
                    </Tab.Pane>
                  </Tab.Content>
                </Col>
                <Col sm={{ span: 3, offset: 1 }}>
                  <Nav variant="pills" className="flex-column text-end">
                    <Nav.Item>
                      <Nav.Link eventKey="project-name">Project Name</Nav.Link>
                    </Nav.Item>
                    <Nav.Item className="danger-zone-nav">
                      <Nav.Link eventKey="danger-zone">Danger Zone</Nav.Link>
                    </Nav.Item>
                  </Nav>
                </Col>
              </Row>
            </Tab.Container>
          </Container>
        </div>
      </Container>
      <ProjectRemoveModal
        project={projectOverview || {}}
        show={removeModelShow}
        onHide={() => setRemoveModelShow(false)}
      />
    </div>
  );
}

export default ProjectSettingsPage;
