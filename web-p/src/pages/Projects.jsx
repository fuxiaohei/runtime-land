import { DefaultLayout } from "../layouts/Layout";
import {
  Button,
  Table,
  Badge,
  Dropdown,
  Spinner,
  Alert,
} from "react-bootstrap";
import { VscLink, VscKebabVertical } from "react-icons/vsc";
import { Link } from "react-router-dom";
import { create_project, list_projects, remove_project } from "../api/projects";
import { useState } from "react";
import { DateTime } from "luxon";
import ProjectRemoveModal from "../components/ProjectRemoveModal";
import ProjectCreateModal from "../components/ProjectCreateModal";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";

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
  const queryClient = useQueryClient();

  const [removeProject, setRemoveProject] = useState({});
  const [showRemoveModal, setShowRemoveModal] = useState(false);
  const [removeAlert, setRemoveAlert] = useState("");

  const {
    isLoading,
    isError,
    error,
    data: projects,
  } = useQuery({
    queryKey: ["projects-list"],
    queryFn: list_projects,
    retry: false,
  });

  const triggerRemove = (data) => {
    console.log("trigger remove", data);
    setShowRemoveModal(true);
    setRemoveProject(data);
  };

  const removeMutation = useMutation({
    mutationFn: (data) => remove_project(data.uuid),
    onSuccess: () => {
      setShowRemoveModal(false);
      setRemoveProject({});
      queryClient.invalidateQueries({ queryKey: ["projects-list"] });
    },
    onError: (error) => {
      setRemoveAlert(error.toString());
    },
  });

  const [showCreateModal, setShowCreateModal] = useState(false);
  const [createAlert, setCreateAlert] = useState("");

  const createMutation = useMutation({
    mutationFn: async (data) => {
      return await create_project(data);
    },
    onSuccess: () => {
      setShowCreateModal(false);
      setCreateAlert("");
      queryClient.invalidateQueries({ queryKey: ["projects-list"] });
    },
    onError: (error) => {
      setCreateAlert(error.toString());
    },
  });

  return (
    <DefaultLayout title="Projects | Runtime.land">
      <div id="projects-header" className="mb-5 mt-3">
        <h3 className="d-inline">Projects</h3>
        <span className="count text-secondary ps-2">
          ({projects?.length || 0})
        </span>
        <Button
          size="sm"
          variant="primary"
          className="create-project-btn ms-5 align-text-bottom"
          onClick={() => setShowCreateModal(true)}
        >
          + New Project
        </Button>
      </div>
      {isLoading ? (
        <Spinner animation="border" />
      ) : (
        <ProjectsTable projects={projects || []} onRemove={triggerRemove} />
      )}
      {isError ? <Alert variant="danger">{error.toString()}</Alert> : null}

      <ProjectCreateModal
        show={showCreateModal}
        onClose={() => setShowCreateModal(false)}
        alert={createAlert}
        onCreate={(data) => createMutation.mutate(data)}
      />
      <ProjectRemoveModal
        show={showRemoveModal}
        data={removeProject}
        onClose={() => {
          setShowRemoveModal(false);
          setRemoveProject({});
        }}
        onRemove={(data) => removeMutation.mutate(data)}
        alert={removeAlert}
      />
    </DefaultLayout>
  );
}

export default ProjectsPage;
