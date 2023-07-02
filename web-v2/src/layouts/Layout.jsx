import Sidebar from "./Sidebar";

function Layout({ children }) {
  return (
    <main className="d-flex flex-nowrap">
      <Sidebar />
      {children}
    </main>
  );
}

export default Layout;
