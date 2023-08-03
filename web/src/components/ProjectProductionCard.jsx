import { Card } from "react-bootstrap";

function ProjectProdutionCard({ deployment }) {
  return (
    <Card className="overview-production-card mb-4">
      <Card.Header>In Production</Card.Header>
      <Card.Body>
        <Card.Text as="div" className="text-secondary">
          <p>The project is live on your production domains.</p>
          <p className="text-uppercase">Domains</p>
          <div className="ms-2">
            <p>
              <a
                href={deployment.prod_url}
                target="_blank"
                className="text-dark fw-bold"
              >
                {deployment.prod_url}
              </a>
            </p>
            <p>
              <a
                href={deployment.domain_url}
                target="_blank"
                className="text-secondary"
              >
                {deployment.domain_url}
              </a>
            </p>
          </div>
        </Card.Text>
      </Card.Body>
    </Card>
  );
}

function ProjectNoProductionCard() {
  return (
    <Card className="overview-no-production-card">
      <Card.Header>No Production</Card.Header>
      <Card.Body>
        <Card.Text as="div" className="pt-3">
          <p>The project is live on your production domains.</p>
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
  );
}

export { ProjectProdutionCard, ProjectNoProductionCard };
