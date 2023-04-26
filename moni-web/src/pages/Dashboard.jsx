import DashboardNavbar from "./DashboardNavbar";
import {
  Container,
  Button,
  Row,
  Col,
  Form,
  InputGroup,
  Card,
} from "react-bootstrap";

function Dashboard() {
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
                  <strong>3</strong> Projects
                </h3>
              </Col>
              <Col id="dashboard-header-right">
                <InputGroup>
                  <Form.Control
                    type="text"
                    placeholder="Search by project name, domain..."
                  />
                  <Button variant="primary">+ New Project</Button>
                </InputGroup>
              </Col>
            </Row>
          </Container>
        </header>
        <section id="dashboard-projects">
          <Container>
            <Row>
              <Col md={6}>
                <Card className="project-card">
                  <Card.Body>
                    <Row>
                      <Col md={8}>
                        <a href="/project">
                          <Card.Title className="project-card-title">
                            quick-trout-fox-22
                          </Card.Title>
                          <Card.Text className="project-card-updated">
                            Updated at 3 hours ago
                          </Card.Text>
                        </a>
                      </Col>
                      <Col md={4} className="project-view">
                        <Button>View</Button>
                      </Col>
                    </Row>
                  </Card.Body>
                </Card>
              </Col>
              <Col md={6}>
                <Card className="project-card">
                  <Card.Body>
                    <Row>
                      <Col md={8}>
                        <Card.Title className="project-card-title">
                          quick-trout-fox-22
                        </Card.Title>
                        <Card.Text className="project-card-updated">
                          Updated at 3 hours ago
                        </Card.Text>
                      </Col>
                      <Col md={4} className="project-view">
                        <Button variant="light" disabled>Not Ready</Button>
                      </Col>
                    </Row>
                  </Card.Body>
                </Card>
              </Col>
            </Row>
            <Row>
              <Col md={6}>
                <Card className="project-card">
                  <Card.Body>
                    <Row>
                      <Col md={8}>
                        <Card.Title className="project-card-title">
                          quick-trout-fox-22
                        </Card.Title>
                        <Card.Text className="project-card-updated">
                          Updated at 3 hours ago
                        </Card.Text>
                      </Col>
                      <Col md={4} className="project-view">
                        <Button>View</Button>
                      </Col>
                    </Row>
                  </Card.Body>
                </Card>
              </Col>
            </Row>
          </Container>
        </section>
      </Container>
    </div>
  );
}

export default Dashboard;
