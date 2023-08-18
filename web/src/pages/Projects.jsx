import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { Button, Container, Form } from "react-bootstrap";
import { createProject, listProjects } from "../api/projects";
import ProjectCreateModal from "../components/ProjectCreateModal";
import ProjectsList from "../components/ProjectsList";
import { AuthProvider } from "../layouts/AuthContext";
import MainLayout from "../layouts/MainLayout";
import LoadingPage from "./Loading";

function ProjectsHeader({ count, onShow, onSearch }) {
  return (
    <div className="projects-header mt-4 d-flex justify-content-between">
      <h2>
        Projects
        <span className="text-secondary fs-5 ms-3 fw-bold">({count})</span>
      </h2>
      <div>
        <Form className="d-inline-block align-middle me-3">
          <Form.Control
            type="search"
            placeholder="Search"
            className="me-2"
            aria-label="Search"
            onChange={(e) => onSearch(e.target.value)}
          />
        </Form>
        <Button variant="primary" onClick={onShow}>
          + New Project
        </Button>
      </div>
    </div>
  );
}

function ProjectsPage() {
  const queryClient = useQueryClient();
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [createAlert, setCreateAlert] = useState("");

  const {
    isLoading,
    isError,
    error,
    data: projects,
  } = useQuery({
    queryKey: ["projects-list"],
    queryFn: listProjects,
    retry: false,
  });

  const createMutation = useMutation({
    mutationFn: async (data) => {
      return await createProject(data);
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

  const [searchFilter, setSearchFilter] = useState("all");

  const handleSearch = (search) => {
    setSearchFilter(search);
  };

  const filterProjects = (projects) => {
    projects = projects || [];
    if (searchFilter === "all") {
      return projects;
    }
    return projects.filter((project) => {
      return project.project.name.includes(searchFilter);
    });
  };

  const renderContainer = (projects) => {
    if (isLoading) {
      return <LoadingPage />;
    }
    if (isError) {
      return <div>{error.toString()}</div>;
    }
    projects = filterProjects(projects);
    return (
      <Container className="mx-auto" id="projects-list-container">
        <ProjectsHeader
          count={projects?.length || 0}
          onShow={() => setShowCreateModal(true)}
          onSearch={handleSearch}
        />
        {projects.length ? (
          <ProjectsList projects={projects || []} />
        ) : (
          <div className="fs-4 mt-4 text-secondary">No projects found.</div>
        )}
        <ProjectCreateModal
          show={showCreateModal}
          handleClose={() => setShowCreateModal(false)}
          handleCreate={(data) => createMutation.mutate(data)}
          alert={createAlert}
        />
      </Container>
    );
  };

  return (
    <AuthProvider>
      <MainLayout title="Projects | Runtime.land">
        {renderContainer(projects)}
      </MainLayout>
    </AuthProvider>
  );
}

export default ProjectsPage;
