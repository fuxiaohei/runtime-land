import { Container } from "react-bootstrap";
import DeploymentsList from "../components/DeploymentsList";
import ProjectHeader from "../components/ProjectHeader";
import { AuthProvider } from "../layouts/AuthContext";
import MainLayout from "../layouts/MainLayout";
import DeploymentProd from "../components/DeploymentProd";
import { useParams } from "react-router-dom";
import { get_overview } from "../api/projects";
import { useQuery } from "@tanstack/react-query";
import LoadingPage from "./Loading";

function ProjectOverviewPage() {
  let { name: projectName } = useParams();

  const {
    isLoading,
    isError,
    error,
    data: overview,
  } = useQuery({
    queryKey: ["project-overview", { projectName }],
    queryFn: async ({ queryKey }) => {
      const { projectName } = queryKey[1];
      const data = await get_overview(projectName);
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
        <ProjectHeader project={overview?.project} activeKey="overview" />
        <DeploymentProd project={overview?.project} />
        <DeploymentsList deployments={overview?.deployments || []} />
      </Container>
    );
  };

  return (
    <AuthProvider>
      <MainLayout>{renderContainer()}</MainLayout>
    </AuthProvider>
  );
}

export default ProjectOverviewPage;
