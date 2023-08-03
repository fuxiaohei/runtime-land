import { Table, Badge } from "react-bootstrap";
import { VscKebabVertical } from "react-icons/vsc";

function ProjectDeploymentsTable({ deployments }) {
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

export default ProjectDeploymentsTable;
