import { Table, Badge } from "react-bootstrap";
import { VscKebabVertical } from "react-icons/vsc";
import { DateTime } from "luxon";

function ProjectDeploymentsTable({ deployments }) {
  const renderLink = (deployment) => {
    let url = deployment.prod_domain
      ? deployment.prod_url
      : deployment.domain_url;
    return (
      <a className="deployment-link text-black" href={url} target="_blank">
        {new URL(url).host}
      </a>
    );
  };

  return (
    <div className="deployments-table-container overflow-y-auto mb-4">
      <Table id="deployments-table" className="mb-0" hover>
        <thead>
          <tr>
            <th>Deployment</th>
            <th style={{ width: "180px" }}>Created At</th>
            <th style={{ width: "5px" }}>Op</th>
          </tr>
        </thead>
        <tbody>
          {deployments.map((deployment) => (
            <tr key={deployment.id}>
              <td className="deployment-name">
                <span className="me-2">{renderLink(deployment)}</span>
                {deployment.prod_domain ? (
                  <Badge className="me-2" bg="success">
                    Prod
                  </Badge>
                ) : null}
              </td>
              <td>
                {DateTime.fromSeconds(deployment.updated_at)
                  .setLocale("en-US")
                  .toFormat("LLL dd, yyyy HH:mm")}
              </td>
              <td></td>
            </tr>
          ))}
        </tbody>
      </Table>
    </div>
  );
}

export default ProjectDeploymentsTable;
