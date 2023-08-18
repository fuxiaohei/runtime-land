import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { Button, Container } from "react-bootstrap";
import { createDeploymentToken, listDeploymentTokens } from "../api/token";
import TokensList from "../components/TokensList";
import { AuthProvider, useAuthContext } from "../layouts/AuthContext";
import MainLayout from "../layouts/MainLayout";
import LoadingPage from "./Loading";

function AccountCard() {
  const { user } = useAuthContext();

  return (
    <div className="account-info mt-3 py-3">
      <div className="d-flex pb-3 justify-content-between border-bottom">
        <div className="d-flex justify-content-start">
          <img
            src={user.avatar_url}
            width="70"
            height="70"
            className="rounded-3"
          />
          <div className="info ms-4">
            <h2 className="fs-3 fw-bold">{user.name}</h2>
            <p className="email text-secondary">{user.email}</p>
          </div>
        </div>
        <div>
          <Button variant="outline-success">Free Plan</Button>
        </div>
      </div>
    </div>
  );
}

function AccountPage() {
  const queryClient = useQueryClient();
  const {
    isLoading,
    isError,
    error,
    data: tokens,
  } = useQuery({
    queryKey: ["tokens"],
    queryFn: listDeploymentTokens,
    retry: false,
  });

  const [newToken, setNewToken] = useState(null);

  const newTokenMutation = useMutation({
    mutationFn: async (name) => {
      return await createDeploymentToken(name);
    },
    onSuccess: (data) => {
      setNewToken(data);
    },
    onError: (error) => {
      console.log(error);
    },
  });

  const handleNewToken = async (token) => {
    await newTokenMutation.mutateAsync(token);
  };

  const handleNewTokenDone = () => {
    setNewToken(null);
    queryClient.invalidateQueries({ queryKey: ["tokens"] });
  };

  const handleRemoveToken = () => {
    queryClient.invalidateQueries({ queryKey: ["tokens"] });
  };

  const renderContainer = () => {
    const tokenContainer = isLoading ? (
      <LoadingPage />
    ) : (
      <TokensList
        tokens={tokens}
        newToken={newToken}
        handleNewToken={handleNewToken}
        handleNewTokenDone={handleNewTokenDone}
        handleRemoveToken={handleRemoveToken}
      />
    );
    return (
      <Container id="account-container">
        <AccountCard />
        {tokenContainer}
      </Container>
    );
  };

  return (
    <AuthProvider>
      <MainLayout title="Account | Runtime.land">
        {renderContainer()}
      </MainLayout>
    </AuthProvider>
  );
}

export default AccountPage;
