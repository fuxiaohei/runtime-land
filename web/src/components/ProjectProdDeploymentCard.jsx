import { Card, ListGroup } from "react-bootstrap";
import { ButtonLink } from "./ButtonLink";
import { BsClock } from "react-icons/bs";
import TimeAgo from "javascript-time-ago";

function ProjectProdDeploymentCard({ deployment }) {
  const timeAgo = new TimeAgo("en-US");
  const listDomains = deployment.domainsList.map((domain, index) => (
    <ListGroup.Item key={domain}>
      <a href={deployment.urlsList[index]} target="_blank">
        {domain}
      </a>
    </ListGroup.Item>
  ));
  return (
    <Card className="project-prod-card mb-3">
      <Card.Body>
        <Card.Title className="d-flex justify-content-between">
          <div>Production Deployment</div>
          <div className="deployment-prod-logs">
            <ButtonLink to="./logs" variant="success" disabled>
              Logs
            </ButtonLink>
          </div>
        </Card.Title>
        <div>
          <ListGroup variant="flush" className="project-prod-domains">
            {listDomains}
            <ListGroup.Item className="text-end small text-muted text-uppercase">
              <BsClock size={10} className="me-1" />
              {timeAgo.format(deployment.updatedAt * 1000)}
            </ListGroup.Item>
          </ListGroup>
        </div>
      </Card.Body>
    </Card>
  );
}

export default ProjectProdDeploymentCard;
