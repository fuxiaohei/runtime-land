import {
  Container,
  Row,
  Col,
  Button,
  Nav,
  Card,
  ListGroup,
} from "react-bootstrap";
import DashboardNavbar from "../components/DashboardNavbar";
import {
  BsClouds,
  BsCheck2Circle,
  BsFillArrowUpLeftSquareFill,
} from "react-icons/bs";

function ProjectPage() {
  return (
    <div>
      <DashboardNavbar />
      <Container id="project-container">
        <header id="project-header">
          <Container>
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
          </Container>
        </header>
        <div id="project-tabs-container">
          <Container>
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
          </Container>
        </div>
        <div id="project-overview-container" className="mt-4">
          <Container>
            <Row>
              <Col lg={9} md={12} id="project-overview-left">
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
                    <ListGroup
                      variant="flush"
                      className="project-deployment-list"
                    >
                      <ListGroup.Item className="lh-lg d-flex justify-content-between">
                        <div className="deployment-metadata text-truncate">
                          <BsCheck2Circle
                            className="status-icon me-2"
                            size={20}
                          />
                          <span className="name">
                            quick-trout-91-3hz5hraraa40.deno.dev
                          </span>
                        </div>
                        <div className="deployment-promotion">
                          <span className="time-ago small text-muted">
                            3 weeks ago
                          </span>
                          <span className="promote-btn">...</span>
                        </div>
                      </ListGroup.Item>
                      <ListGroup.Item className="lh-lg">
                        Dapibus ac facilisis in
                      </ListGroup.Item>
                      <ListGroup.Item className="lh-lg">
                        Morbi leo risus
                      </ListGroup.Item>
                      <ListGroup.Item className="lh-lg">
                        Porta ac consectetur ac
                      </ListGroup.Item>
                    </ListGroup>
                  </Card.Body>
                </Card>
              </Col>
              <Col
                lg={3}
                md="auto"
                className="d-none d-lg-block d-xl-block d-xxl-block"
                id="project-overview-right"
              >
                right
              </Col>
            </Row>
          </Container>
        </div>
      </Container>
    </div>
  );
}

export default ProjectPage;
