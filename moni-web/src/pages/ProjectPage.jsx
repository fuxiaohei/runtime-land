import { Container, Row, Col, Button, Nav } from "react-bootstrap";
import DashboardNavbar from "../components/DashboardNavbar";
import { BsClouds, BsFillArrowUpLeftSquareFill } from "react-icons/bs";

function ProjectPage() {
  return (
    <div>
      <DashboardNavbar />
      <Container id="project-container">
        <header id="project-header">
          <Container>
            <Row>
              <Col md={4} id="project-header-left">
                <h2>dry-toad-81</h2>
                <p>Github / Pending</p>
              </Col>
              <Col id="project-header-right">
                <Button variant="secondary" size="sm" href="/dashboard">
                  <BsFillArrowUpLeftSquareFill size={16} className="icon" />
                  Projects
                </Button>
                <Button variant="primary" size="sm">
                  <BsClouds size={16} className="icon" />
                  View
                </Button>
              </Col>
            </Row>
          </Container>
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
      </Container>
    </div>
  );
}

export default ProjectPage;
