import { BsClouds, BsFillArrowUpLeftSquareFill } from "react-icons/bs";
import { Container, Row, Col, Button } from "react-bootstrap";

function ProjectHeader({ projectName }) {
  return (
    <header id="project-header">
      <Container>
        <Row>
          <Col md={6} sm={6} xs={5} id="project-header-left">
            <h2>{projectName}</h2>
            <p>Only preview deployment</p>
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
  );
}

export default ProjectHeader;
