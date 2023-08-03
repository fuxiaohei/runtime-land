import { Button, Container, Form, InputGroup } from "react-bootstrap";
import ProjectHeader from "../components/ProjectHeader";
import { AuthProvider } from "../layouts/AuthContext";
import MainLayout from "../layouts/MainLayout";
import { useParams } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { get_project } from "../api/projects";
import LoadingPage from "./Loading";

function ProjectSettingPage() {
  let { name: projectName } = useParams();

  const {
    isLoading,
    isError,
    error,
    data: project,
  } = useQuery({
    queryKey: ["project-simple", { projectName }],
    queryFn: async ({ queryKey }) => {
      const { projectName } = queryKey[1];
      const data = await get_project(projectName);
      return data;
    },
    retry: false,
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
            <Form>
              <InputGroup className="mb-3 project-name-input">
                <Form.Control placeholder="the name of your project" />
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
            <Button className="mb-3" variant="outline-danger">
              Delete
            </Button>
          </div>
        </div>
      </Container>
    );
  };

  return (
    <AuthProvider>
      <MainLayout>{renderContainer()}</MainLayout>
    </AuthProvider>
  );
}

export default ProjectSettingPage;
