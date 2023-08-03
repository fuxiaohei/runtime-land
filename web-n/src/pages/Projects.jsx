import { Button, Container, Form } from "react-bootstrap";
import ProjectsList from "../components/ProjectsList";
import { useState } from "react";
import ProjectCreateModal from "../components/ProjectCreateModal";
import { AuthProvider } from "../layouts/AuthContext";
import MainLayout from "../layouts/MainLayout";
import { create_project, list_projects } from "../api/projects";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import LoadingPage from "./Loading";

function ProjectsHeader({ count, onShow }) {
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
    queryFn: list_projects,
    retry: false,
  });

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

  const renderContainer = () => {
    if (isLoading) {
      return <LoadingPage />;
    }
    if (isError) {
      return <div>Error: {error.toString()}</div>;
    }
    return (
      <Container className="mx-auto" id="projects-list-container">
        <ProjectsHeader
          count={projects?.length || 0}
          onShow={() => setShowCreateModal(true)}
        />
        <ProjectsList projects={projects || []} />
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
      <MainLayout>{renderContainer()}</MainLayout>
    </AuthProvider>
  );
}

export default ProjectsPage;
