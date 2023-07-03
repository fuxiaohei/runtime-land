import {
  Container,
  Table,
  Row,
  Col,
  Card,
  Badge,
  Button,
} from "react-bootstrap";
import { ProjectLayout } from "../layouts/Layout";
import { LiaExternalLinkAltSolid } from "react-icons/lia";
import { Link } from "react-router-dom";
import { VscLink, VscKebabVertical } from "react-icons/vsc";

function DeploymentsTable() {
  return (
    <div className="deployments-table-container mb-4">
      <Table id="deployments-table" className="mb-0" hover>
        <thead>
          <tr>
            <th>Deployment</th>
            <th style={{ width: "180px" }}>Created At</th>
            <th style={{ width: "200px" }}>Op</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td className="deployment-name fw-bold">
              <span className="me-2">
                <a href="https://quick-trout-91-m9t91f4cbqh0.deno.dev">
                  quick-trout-91-m9t91f4cbqh0.deno.dev
                </a>
              </span>
            </td>
            <td>Jul 2, 2023 18:29</td>
            <td>op</td>
          </tr>
          <tr>
            <td className="deployment-name fw-bold">
              <span className="me-2">
                <a
                  className="text-dark"
                  href="/projects/polite-pike-746/overview"
                >
                  quick-dog-91.deno.dev
                </a>
              </span>
              <Badge className="me-2" bg="success">
                Prod
              </Badge>
            </td>
            <td>Jul 2, 2023 18:29</td>
            <td>
              <VscKebabVertical />
            </td>
          </tr>
        </tbody>
      </Table>
    </div>
  );
}

function ProjectOverviewPage() {
  return (
    <ProjectLayout projectName="polite-pike-746">
      <div id="overview-header" className="mb-5 mt-3">
        <h3 className="d-inline">polite-pike-746</h3>
        <span className="prod-link text-secondary ps-3">
          <Button
            size="sm"
            variant="success"
            href="#"
            className="align-text-bottom"
          >
            <LiaExternalLinkAltSolid className="me-1" />
            View
          </Button>
        </span>
      </div>
      <Container fluid className="p-0">
        <Row>
          <Col md={3}>
            <Card className="overview-production-card mb-4">
              <Card.Header>In Production</Card.Header>
              <Card.Body>
                <Card.Text as="div" className="text-secondary">
                  <p>The project is live on your production domains.</p>
                  <p className="text-uppercase">Domains</p>
                  <div className="ms-2">
                    <p>
                      <a href="#" className="text-dark fw-bold">
                        https://polite-pike-746.netlify.app
                      </a>
                    </p>
                    <p>
                      <a href="#" className="text-secondary">
                        https://polite-pike-746-3hz5hraree40.netlify.app
                      </a>
                    </p>
                  </div>
                </Card.Text>
              </Card.Body>
            </Card>
            <Card className="overview-no-production-card">
              <Card.Header>No Production</Card.Header>
              <Card.Body>
                <Card.Title className="text-secondary">
                  The project is live on your production domains.
                </Card.Title>
                <Card.Text as="div" className="pt-3">
                  <p>
                    <a href="#" className="text-dark fw-bold">
                      https://polite-pike-746.netlify.app
                    </a>
                  </p>
                  <p>
                    <a href="#" className="text-secondary">
                      {" "}
                      https://polite-pike-746-3hz5hraree40.netlify.app
                    </a>
                  </p>
                </Card.Text>
              </Card.Body>
            </Card>
          </Col>
          <Col md={8}>
            <Card>
              <Card.Header>Deployments</Card.Header>
              <Card.Body>
                <Card.Text as="div">
                  <p className="text-secondary">
                    All deployments of this project.
                  </p>
                </Card.Text>
                <DeploymentsTable />
              </Card.Body>
            </Card>
          </Col>
        </Row>
      </Container>
    </ProjectLayout>
  );
}

export default ProjectOverviewPage;
