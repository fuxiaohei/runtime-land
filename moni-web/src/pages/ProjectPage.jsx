import { Container, Row, Col, Button, Nav, Card } from "react-bootstrap";
import DashboardNavbar from "../components/DashboardNavbar";
import { BsClouds, BsFillArrowUpLeftSquareFill } from "react-icons/bs";

function ProjectPage() {
  return (
    <div>
      <DashboardNavbar />
      <Container id="project-container">
        <header id="project-header">
          <Row>
            <Col md={4} sm={4} xs={4} id="project-header-left">
              <h2>dry-toad-81</h2>
              <p>Github / Pending</p>
            </Col>
            <Col id="project-header-right">
              <Button variant="secondary" size="sm" href="/projects">
                <BsFillArrowUpLeftSquareFill size={16} className="icon" />
                Projects
              </Button>
              <Button variant="primary" size="sm">
                <BsClouds size={16} className="icon" />
                View
              </Button>
            </Col>
          </Row>
        </header>
        <div id="project-tabs-container">
          <Nav activeKey="link-overview">
            <Nav.Item>
              <Nav.Link eventKey="link-overview" href="#overview">
                Overview
              </Nav.Link>
            </Nav.Item>
            <Nav.Item>
              <Nav.Link eventKey="link-1" href="#link-1">
                Deployments
              </Nav.Link>
            </Nav.Item>
            <Nav.Item>
              <Nav.Link eventKey="link-2" href="#link-2">
                Settings
              </Nav.Link>
            </Nav.Item>
            <Nav.Item>
              <Nav.Link eventKey="disabled" disabled>
                Logs
              </Nav.Link>
            </Nav.Item>
            <Nav.Item>
              <Nav.Link eventKey="disabled" disabled>
                Analytics
              </Nav.Link>
            </Nav.Item>
          </Nav>
        </div>
        <div id="project-overview-container" className="mt-4">
          <Row>
            <Col md={9} id="project-overview-left">
              <Card className="project-no-prod-card mb-3">
                <Card.Body>
                  <Card.Title>No Prodution Deployment</Card.Title>
                  <p className="text-muted">
                    Promote a preview deployment to production to get started,{" "}
                    <br />
                    or use <span>moni-cli deploy --prodution</span> to deploy
                    directly from your local project.
                  </p>
                </Card.Body>
              </Card>
              <Card className="project-deployment-card">
                <Card.Body>
                  <Card.Title>Latest Deployments</Card.Title>
                </Card.Body>
              </Card>
            </Col>
            <Col md={3} id="project-overview-right">
              right
            </Col>
          </Row>
        </div>
      </Container>
    </div>
  );
}

export default ProjectPage;
