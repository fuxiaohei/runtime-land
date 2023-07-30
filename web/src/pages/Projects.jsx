import { DefaultLayout } from "../layouts/Layout";
import { Button, Table, Badge, Dropdown } from "react-bootstrap";
import { VscLink, VscKebabVertical } from "react-icons/vsc";
import { Link } from "react-router-dom";
import { create_project, list_projects, remove_project } from "../api/projects";
import { useEffect, useState } from "react";
import { DateTime } from "luxon";
import ProjectRemoveModal from "../components/ProjectRemoveModal";
import ProjectCreateModal from "../components/ProjectCreateModal";

function ProjectsTable({ projects, onRemove }) {
  const renderProjectRow = (project) => {
    let data = project.project;

    const handleRemove = async (event) => {
      event.preventDefault();
      await onRemove(data);
    };

    return (
      <tr role="button" key={data.uuid}>
        <td className="project-name fw-bold">
          <span className="me-2">
            <Link
              className="text-dark"
              to={"/projects/" + data.name + "/overview"}
            >
              {data.name}
            </Link>
          </span>
          {project.prod_deployment ? (
            <>
              <Badge className="me-2" bg="success">
                Prod
              </Badge>
              <a
                className="text-success"
                href={project.prod_deployment.prod_url}
                target="_blank"
              >
                <VscLink />
              </a>
            </>
          ) : null}
          {data.status === "pending" ? (
            <Badge className="me-2" bg="secondary">
              Pending
            </Badge>
          ) : null}
        </td>
        <td>{data.language}</td>
        <td className="deploy-count fw-bold">{project.deployments_count}</td>
        <td>
          {DateTime.fromSeconds(data.updated_at)
            .setLocale("en-US")
            .toLocaleString(DateTime.DATE_FULL)}
        </td>
        <td className="ops">
          <Dropdown align="end">
            <Dropdown.Toggle as="span">
              <VscKebabVertical />
            </Dropdown.Toggle>

            <Dropdown.Menu className="py-0 overflow-hidden">
              <Dropdown.Item as="span">
                <Link
                  className="text-dark text-decoration-none"
                  to={"/projects/" + data.name + "/settings"}
                >
                  Settings
                </Link>
              </Dropdown.Item>
              <Dropdown.Item disabled>Traffic</Dropdown.Item>
              {data.status === "pending" ? (
                <Dropdown.Item className="text-danger" onClick={handleRemove}>
                  Remove
                </Dropdown.Item>
              ) : null}
            </Dropdown.Menu>
          </Dropdown>
        </td>
      </tr>
    );
  };

  return (
    <div className="project-table-container rounded overflow-hidden border me-4">
      <Table id="projects-table" className="mb-0" hover>
        <thead>
          <tr>
            <th>Name</th>
            <th style={{ width: "120px" }}>Language</th>
            <th style={{ width: "100px" }}>Deploys</th>
            <th style={{ width: "180px" }}>Last Updated</th>
            <th style={{ width: "50px" }}></th>
          </tr>
        </thead>
        <tbody>{projects.map((project) => renderProjectRow(project))}</tbody>
      </Table>
    </div>
  );
}

function ProjectsPage() {
  const [projects, setProjects] = useState([]);
  const [removeProject, setRemoveProject] = useState({});
  const [showRemoveModal, setShowRemoveModal] = useState(false);
  const [removeAlert, setRemoveAlert] = useState("");

  const fetchProjects = async () => {
    let response = await list_projects();
    if (response.error) {
      console.log(response.error);
      return;
    }
    setProjects(response);
  };

  useEffect(() => {
    if (projects.length === 0) {
      fetchProjects();
    }
  }, []);

  const triggerRemove = (data) => {
    console.log("trigger remove", data);
    setShowRemoveModal(true);
    setRemoveProject(data);
  };

  const handleRemove = async (data) => {
    console.log("handle remove", data);
    let response = await remove_project(data.uuid);
    if (response.error) {
      setRemoveAlert(response.error);
      return;
    }
    setShowRemoveModal(false);
    setRemoveProject({});
    fetchProjects();
  };

  const [showCreateModal, setShowCreateModal] = useState(false);
  const [createAlert, setCreateAlert] = useState("");

  const handleCreate = async (data) => {
    console.log("handle create", data);
    let response = await create_project(data);
    if (response.error) {
      setCreateAlert(response.error);
      return;
    }
    setShowCreateModal(false);
    setCreateAlert("");
    fetchProjects();
  };

  return (
    <DefaultLayout title="Projects | Runtime.land">
      <div id="projects-header" className="mb-5 mt-3">
        <h3 className="d-inline">Projects</h3>
        <span className="count text-secondary ps-2">({projects.length})</span>
        <Button
          size="sm"
          variant="primary"
          className="create-project-btn ms-5 align-text-bottom"
          onClick={() => setShowCreateModal(true)}
        >
          + New Project
        </Button>
      </div>
      <ProjectsTable projects={projects || []} onRemove={triggerRemove} />
      <ProjectCreateModal
        show={showCreateModal}
        onClose={() => setShowCreateModal(false)}
        alert={createAlert}
        onCreate={handleCreate}
      />
      <ProjectRemoveModal
        show={showRemoveModal}
        data={removeProject}
        onClose={() => {
          setShowRemoveModal(false);
          setRemoveProject({});
        }}
        onRemove={handleRemove}
        alert={removeAlert}
      />
    </DefaultLayout>
  );
}

export default ProjectsPage;
