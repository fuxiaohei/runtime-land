import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { Alert, Container } from "react-bootstrap";
import { listStorageSettings, updateStorageSettings } from "../../api/regions";
import AdminNavHeader from "../../components/AdminNavHeader";
import AdminStorageForm from "../../components/AdminStorageForm";
import { AuthProvider } from "../../layouts/AuthContext";
import MainLayout from "../../layouts/MainLayout";
import QueryWrapper from "../../layouts/QueryWrapper";

function AdminStoragePage() {
  const queryClient = useQueryClient();
  const {
    isLoading,
    isError,
    error,
    data: settings,
  } = useQuery({
    queryKey: ["settings-storage"],
    queryFn: listStorageSettings,
    retry: false,
  });

  const [isSuccess, setSuccess] = useState(false);

  const updateMutation = useMutation({
    mutationFn: updateStorageSettings,
    onSuccess: () => {
      queryClient.invalidateQueries("settings-storage");
      setSuccess(true);
    },
    onError: (error) => {},
  });

  const handleSubmit = (data) => {
    setSuccess(false);
    updateMutation.mutate(data);
  };

  return (
    <AuthProvider>
      <MainLayout title="Storage | Admin Panel | Runtime.land">
        <Container id="admin-page" className="mt-4">
          <h3 className="mb-3">Admin Panel</h3>
          <AdminNavHeader activeKey="storage" />
          <p className="text-secondary py-2">
            Storage settings are used to configure the storage provider that
            Runtime.land uses to store your project's files.
          </p>
          <Alert variant="warning">
            All assets are not migrated to the new storage provider yet. Please
            do not change the storage provider until your migration is
            completed.
          </Alert>
          <QueryWrapper isLoading={isLoading} isError={isError} error={error}>
            <AdminStorageForm
              isSuccess={isSuccess}
              data={settings}
              onSubmit={handleSubmit}
            />
          </QueryWrapper>
        </Container>
      </MainLayout>
    </AuthProvider>
  );
}

export default AdminStoragePage;
