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
  OverlayTrigger,
  Tooltip,
} from "react-bootstrap";
import { BiRefresh } from "react-icons/bi";

function NewProjectPage() {
  return (
    <div>
      <DashboardNavbar />
      <Container id="dashboard-container">
        <header id="new-project-header">
          <h2>New Project</h2>
          <h3>
            Create a new project by entering the project name and selecting the
            template.
          </h3>
        </header>
        <Container id="new-project-cards">
          <Row>
            <Col md={5}>
              <Card id="new-project-container">
                <Card.Body>
                  <Card.Title>Project Name</Card.Title>
                  <div className="project-name-div">
                    <p>
                      <InputGroup>
                        <Form.Control
                          type="text"
                          placeholder="Enter your project name"
                        />
                        <OverlayTrigger
                          placement="top"
                          delay={{ show: 0, hide: 200 }}
                          overlay={<Tooltip>Regenerate project name</Tooltip>}
                        >
                          <Button variant="dark">
                            <BiRefresh />
                          </Button>
                        </OverlayTrigger>
                      </InputGroup>
                    </p>
                    <p className="fs-6 text-muted">
                      Edit and deploy directly from a local project using
                      moni-cli.
                    </p>
                    <p className="text-end">
                      <Button>Create Empty Project</Button>
                    </p>
                  </div>
                </Card.Body>
              </Card>
            </Col>
            <Col md={7}>
              <Card id="project-template-container">
                <Card.Body>
                  <Card.Title className="d-flex justify-content-between">
                    <div className="title">
                      <h3>Template</h3>
                      <p>Build with examples</p>
                    </div>
                    <div className="btn">
                      <Button variant="light">Create</Button>
                    </div>
                  </Card.Title>
                  <hr />
                </Card.Body>
              </Card>
            </Col>
          </Row>
        </Container>
      </Container>
    </div>
  );
}

export default NewProjectPage;
