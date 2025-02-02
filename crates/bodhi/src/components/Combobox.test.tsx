import { ComboBoxResponsive } from "@/components/Combobox";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { vi } from "vitest";

// Mock useMediaQuery hook
const useMediaQuery = vi.fn();

vi.mock("@/hooks/use-media-query", () => ({
  useMediaQuery: (query: string) => {
    return useMediaQuery(query);
  }
}));

// Mock required HTMLElement methods and styles for Radix UI and Vaul components
Object.assign(window.HTMLElement.prototype, {
  scrollIntoView: vi.fn(),
  releasePointerCapture: vi.fn(),
  hasPointerCapture: vi.fn(),
  setPointerCapture: vi.fn(),
  getBoundingClientRect: vi.fn().mockReturnValue({
    x: 0,
    y: 0,
    width: 0,
    height: 0,
    top: 0,
    right: 0,
    bottom: 0,
    left: 0,
  }),
});

const mockStatuses = [
  { value: "active", label: "Active" },
  { value: "inactive", label: "Inactive" },
  { value: "pending", label: "Pending" }
]

describe("ComboBoxResponsive", () => {
  const defaultProps = {
    selectedStatus: null,
    setSelectedStatus: vi.fn(),
    statuses: mockStatuses,
    placeholder: "Select status"
  }

  describe("Desktop View", () => {
    beforeEach(() => {
      // Mock desktop view
      vi.mocked(useMediaQuery).mockReturnValue(true);
    });

    it("renders with placeholder when no status is selected", () => {
      render(<ComboBoxResponsive {...defaultProps} />);

      expect(screen.getByRole("combobox")).toHaveTextContent("Select status");
    });

    it("renders with selected status label", () => {
      render(
        <ComboBoxResponsive
          {...defaultProps}
          selectedStatus={{ value: "active", label: "Active" }}
        />
      );

      expect(screen.getByRole("combobox")).toHaveTextContent("Active");
    });

    it("opens popover and allows status selection", async () => {
      const user = userEvent.setup();
      render(<ComboBoxResponsive {...defaultProps} />);

      // Click to open popover
      await user.click(screen.getByRole("combobox"));

      // Verify options are displayed
      const options = screen.getAllByRole("option");
      expect(options).toHaveLength(mockStatuses.length);

      // Select an option
      await user.click(options[0]);

      // Verify setSelectedStatus was called with correct value
      expect(defaultProps.setSelectedStatus).toHaveBeenCalledWith(mockStatuses[0]);
    });

    it("filters options based on search input", async () => {
      const user = userEvent.setup();
      render(<ComboBoxResponsive {...defaultProps} />);

      // Open popover
      await user.click(screen.getByRole("combobox"));

      // Type in search input
      const searchInput = screen.getByPlaceholderText("Filter ...");
      await user.type(searchInput, "act");

      // Verify filtered options
      const options = screen.getAllByRole("option");
      expect(options).toHaveLength(2);
      expect(options[0]).toHaveTextContent("Active");
      expect(options[1]).toHaveTextContent("Inactive");
    });
  });

  describe("Mobile View", () => {
    beforeEach(() => {
      // Mock mobile view
      vi.mocked(useMediaQuery).mockReturnValue(false);
    });

    it("renders drawer on mobile", async () => {
      const user = userEvent.setup();
      render(<ComboBoxResponsive {...defaultProps} />);

      // Click to open drawer
      await user.click(screen.getByRole("combobox"));

      // Verify drawer content is rendered
      expect(screen.getByRole("dialog")).toBeInTheDocument();
    });

    test.skip("allows status selection in drawer", async () => {
      const user = userEvent.setup();
      render(<ComboBoxResponsive {...defaultProps} />);

      // Open drawer
      await user.click(screen.getByRole("combobox"));

      // Find and click option within drawer
      const dialog = screen.getByRole("dialog");
      const options = within(dialog).getAllByRole("option");
      await user.click(options[0]);

      // Verify setSelectedStatus was called
      expect(defaultProps.setSelectedStatus).toHaveBeenCalledWith(mockStatuses[0]);
    });

    test.skip("filters options in drawer", async () => {
      const user = userEvent.setup();
      render(<ComboBoxResponsive {...defaultProps} />);

      // Open drawer
      await user.click(screen.getByRole("combobox"));

      // Type in search input
      const dialog = screen.getByRole("dialog");
      const searchInput = within(dialog).getByPlaceholderText("Filter status...");
      await user.type(searchInput, "pend");

      // Verify filtered options
      const options = within(dialog).getAllByRole("option");
      expect(options).toHaveLength(1);
      expect(options[0]).toHaveTextContent("Pending");
    })
  })
});
