import { createProject } from "../cloud/projects";
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
  Alert,
} from "react-bootstrap";
import { BiRefresh } from "react-icons/bi";
import {
  uniqueNamesGenerator,
  NumberDictionary,
  adjectives,
  colors,
} from "unique-names-generator";
import React from "react";
import { useNavigate } from "react-router-dom";

function NewProjectPage() {
  const generateName = () => {
    const numberDictionary = NumberDictionary.generate({ min: 100, max: 999 });
    const shortName = uniqueNamesGenerator({
      dictionaries: [adjectives, colors, numberDictionary],
      length: 3,
      separator: "-",
    });
    return shortName;
  };
  const [autoName, setAutoName] = React.useState(generateName());
  const [showAlert, setShowAlert] = React.useState(false);
  const [alertMessage, setAlertMessage] = React.useState("");
  const navigate = useNavigate();

  const handleRefreshGenerate = async (event) => {
    setAutoName(generateName());
  };

  const handleSubmitNewProject = async (event) => {
    if (!autoName) {
      setShowAlert(true);
      setAlertMessage("Project name cannot be empty");
      return;
    }
    setShowAlert(false);
    let response = await createProject(autoName, "rust"); // only support rust now
    if (response.error) {
      setShowAlert(true);
      setAlertMessage(response.error);
      return;
    }
    navigate(`/projects/${autoName}`);
  };

  const renderAlert = () => {
    if (showAlert) {
      return <Alert variant="danger">{alertMessage}</Alert>;
    }
    return null;
  };

  return (
    <div>
      <DashboardNavbar />
      <Container id="dashboard-container">
        <Container id="new-project-top">
          <Container className="px-0">
            <header id="new-project-header">
              <h2>New Project</h2>
              <h3>
                Create a new project by entering the project name and selecting
                the template.
              </h3>
            </header>
          </Container>
          <Container id="new-project-cards">
            <Row>
              <Col md={5} sm={12}>
                <Card id="new-project-container">
                  <Card.Body>
                    <Card.Title>Project Name</Card.Title>
                    <div className="project-name-div">
                      <div className="mb-3">
                        <InputGroup>
                          <Form.Control
                            type="text"
                            placeholder="Enter your project name"
                            value={autoName}
                            onChange={(event) =>
                              setAutoName(event?.target.value)
                            }
                          />
                          <OverlayTrigger
                            placement="top"
                            delay={{ show: 0, hide: 200 }}
                            overlay={<Tooltip>Regenerate project name</Tooltip>}
                          >
                            <Button
                              variant="dark"
                              onClick={handleRefreshGenerate}
                            >
                              <BiRefresh />
                            </Button>
                          </OverlayTrigger>
                        </InputGroup>
                      </div>
                      {renderAlert()}
                      <p className="fs-6 ms-2 text-muted">
                        Edit and deploy directly from a local project using
                        land-cli.
                      </p>
                      <p className="text-end">
                        <Button onClick={handleSubmitNewProject}>
                          Create Empty Project
                        </Button>
                      </p>
                    </div>
                  </Card.Body>
                </Card>
              </Col>
              <Col
                md={7}
                sm="auto"
                className="d-none d-md-block d-lg-block d-xl-block d-xxl-block"
              >
                <Card id="project-template-container">
                  <Card.Body>
                    <Card.Title className="d-flex justify-content-between">
                      <div className="title">
                        <h3>Template</h3>
                        <p>Build with examples</p>
                      </div>
                      <div className="btn">
                        <Button variant="light" disabled>
                          Create
                        </Button>
                      </div>
                    </Card.Title>
                    <hr />
                    <div>Comming soon...</div>
                  </Card.Body>
                </Card>
              </Col>
            </Row>
          </Container>
        </Container>
      </Container>
    </div>
  );
}

export default NewProjectPage;
