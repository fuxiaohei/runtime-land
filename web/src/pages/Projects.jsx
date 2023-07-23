import { DefaultLayout } from "../layouts/Layout";
import { Button, Table, Badge } from "react-bootstrap";
import { VscLink, VscKebabVertical } from "react-icons/vsc";
import { Link } from "react-router-dom";

function ProjectsTable() {
  return (
    <div className="project-table-container rounded overflow-hidden border me-4">
      <Table id="projects-table" className="mb-0" hover>
        <thead>
          <tr>
            <th>Name</th>
            <th style={{ width: "120px" }}>Language</th>
            <th style={{ width: "180px" }}>Region</th>
            <th style={{ width: "50px" }}>De</th>
            <th style={{ width: "180px" }}>Last Updated</th>
            <th style={{ width: "50px" }}>Op</th>
          </tr>
        </thead>
        <tbody>
          <tr role="button">
            <td className="project-name fw-bold">
              <span className="me-2">
                <Link
                  className="text-dark"
                  to="/projects/polite-pike-746/overview"
                >
                  polite-pike-746
                </Link>
              </span>
              <Badge className="me-2" bg="success">
                Prod
              </Badge>
              <a className="text-success" href="#">
                <VscLink />
              </a>
            </td>
            <td>Mark</td>
            <td>US West (Oregon)</td>
            <td className="deploy-count">4</td>
            <td>Jul 2, 2023 18:29</td>
            <td>
              <VscKebabVertical />
            </td>
          </tr>
          <tr role="button">
            <td className="project-name fw-bold">
              <span className="me-2">
                <Link
                  className="text-dark"
                  to="/projects/polite-pike-746/overview"
                >
                  dry-toad-chip-33
                </Link>
              </span>
              <Badge className="me-2" bg="success">
                Prod
              </Badge>
              <a className="text-success" href="#">
                <VscLink />
              </a>
            </td>
            <td>Mark</td>
            <td>US West (Oregon)</td>
            <td className="deploy-count">4</td>
            <td>Jul 2, 2023 18:29</td>
            <td>
              <VscKebabVertical />
            </td>
          </tr>
          <tr role="button">
            <td className="project-name fw-bold">
              <span className="me-2">
                <Link
                  className="text-dark"
                  to="/projects/polite-pike-746/overview"
                >
                  proud-happy-39
                </Link>
              </span>
            </td>
            <td>Mark</td>
            <td>US West (Oregon)</td>
            <td className="deploy-count">4</td>
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

function ProjectsPage() {
  return (
    <DefaultLayout title="Projects | Runtime.land">
      <div id="projects-header" className="mb-5 mt-3">
        <h3 className="d-inline">Projects</h3>
        <span className="count text-secondary ps-2">(2)</span>
        <Button
          size="sm"
          variant="primary"
          className="create-project-btn ms-5 align-text-bottom"
        >
          + New Project
        </Button>
      </div>
      <ProjectsTable />
    </DefaultLayout>
  );
}

export default ProjectsPage;
