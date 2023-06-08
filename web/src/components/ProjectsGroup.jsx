import { Link } from "react-router-dom";
import { Container, Button, Row, Col, Card } from "react-bootstrap";
import TimeAgo from "javascript-time-ago";

function ProjectsGroup({ projects }) {
  // separate projects array to two-item pairs list
  const projectLists = projects.data.reduce((resultArray, item, index) => {
    const chunkIndex = Math.floor(index / 2);
    if (!resultArray[chunkIndex]) {
      resultArray[chunkIndex] = [];
    }
    resultArray[chunkIndex].push(item);
    return resultArray;
  }, []);

  const projectPairItem = (projectPair) => {
    const timeAgo = new TimeAgo("en-US");
    return projectPair.map((project) => (
      <Col md={6} key={project.uuid}>
        <Card className="project-card">
          <Card.Body>
            <Row>
              <Col md={8}>
                <Link to={"/projects/" + project.name}>
                  <Card.Title className="project-card-title">
                    {project.name}
                  </Card.Title>
                  <Card.Text className="project-card-updated">
                    Updated at {timeAgo.format(project.updatedAt * 1000)}
                  </Card.Text>
                </Link>
              </Col>
              <Col md={4} className="project-view">
                {project.prodDeployment ? (
                  <Button>View</Button>
                ) : (
                  <Button variant="light">Dev</Button>
                )}
              </Col>
            </Row>
          </Card.Body>
        </Card>
      </Col>
    ));
  };

  const projectListItems = projectLists.map((projectList, index) => {
    let projectPair = projectPairItem(projectList);
    return <Row key={"project-" + index}>{projectPair}</Row>;
  });

  return <Container>{projectListItems}</Container>;
}

export default ProjectsGroup;
