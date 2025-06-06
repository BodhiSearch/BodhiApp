import { UsersPageContent } from "@/components/users/UsersPage";
import { render, screen } from "@testing-library/react";

describe("UsersPageContent", () => {
  it("renders the coming soon message", () => {
    render(<UsersPageContent />);

    expect(screen.getByText("Coming Soon")).toBeInTheDocument();
    expect(
      screen.getByText("We're working hard to bring you these amazing features. Thanks and Stay Tuned!")
    ).toBeInTheDocument();
  });
});
