import { useMutation, useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { Button, Container, Form, InputGroup } from "react-bootstrap";
import { useNavigate, useParams } from "react-router-dom";
import { getProject, removeProject, renameProject } from "../api/projects";
import ProjectHeader from "../components/ProjectHeader";
import ProjectRemoveModal from "../components/ProjectRemoveModal";
import { AuthProvider } from "../layouts/AuthContext";
import MainLayout from "../layouts/MainLayout";
import LoadingPage from "./Loading";

function ProjectSettingPage() {
  const { name: projectName } = useParams();
  const navigate = useNavigate();
  const [showRemoveModal, setShowRemoveModal] = useState(false);
  const [removeAlert, setRemoveAlert] = useState("");
  const [inputProjectName, setInputProjectName] = useState(projectName);

  const {
    isLoading,
    isError,
    error,
    data: project,
  } = useQuery({
    queryKey: ["project-simple", { projectName }],
    queryFn: async ({ queryKey }) => {
      const { projectName } = queryKey[1];
      const data = await getProject(projectName);
      return data;
    },
    retry: false,
  });

  const removeMutation = useMutation({
    mutationFn: async () => {
      return await removeProject(project.uuid);
    },
    onSuccess: () => {
      setShowRemoveModal(false);
      navigate("/projects");
    },
    onError: (error) => {
      setRemoveAlert(error.toString());
    },
  });

  const handleSubmit = async (e) => {
    e.preventDefault();
    if (projectName === inputProjectName) {
      return;
    }
    renameMutation.mutate({
      old_name: projectName,
      new_name: inputProjectName,
    });
  };

  const renameMutation = useMutation({
    mutationFn: async ({ old_name, new_name }) => {
      return await renameProject(old_name, new_name);
    },
    onSuccess: async () => {
      navigate("/projects/" + inputProjectName + "/setting");
    },
    onError: (error) => {},
  });

  const renderContainer = () => {
    if (isLoading) {
      return <LoadingPage />;
    }

    return (
      <Container className="mx-auto" id="project-overview-container">
        <ProjectHeader project={project} activeKey="setting" />
        <div className="py-3 border-top">
          <div className="project-name-updater border-bottom">
            <h5 className="fw-bold">Project Name</h5>
            <p className="text-secondary">The name of your project.</p>
            <Form onSubmit={handleSubmit}>
              <InputGroup className="mb-3 project-name-input">
                <Form.Control
                  defaultValue={inputProjectName}
                  required
                  placeholder="the name of your project"
                  onChange={(event) => setInputProjectName(event.target.value)}
                />
                <InputGroup.Text>.{project.subdomain}</InputGroup.Text>
              </InputGroup>
              <Button className="mb-3" variant="primary" type="submit">
                Save
              </Button>
            </Form>
          </div>
          <div className="project-remove-zone mt-3">
            <h5 className="mb-3 fw-bold text-danger">Delete Project</h5>
            <p className="text-secondary">
              Delete <strong>{project.name}</strong> and all of its deployments.
              Be careful, this action cannot be undone.
            </p>
            <Button
              className="mb-3"
              variant="outline-danger"
              onClick={() => setShowRemoveModal(true)}
            >
              Delete
            </Button>
          </div>
        </div>
      </Container>
    );
  };

  return (
    <AuthProvider>
      <MainLayout title="Project Settings | Runtime.land">
        {renderContainer()}
        <ProjectRemoveModal
          show={showRemoveModal}
          handleClose={() => setShowRemoveModal(false)}
          alert={removeAlert}
          handleRemove={() => removeMutation.mutate()}
        />
      </MainLayout>
    </AuthProvider>
  );
}

export default ProjectSettingPage;
