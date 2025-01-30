export function showSuccessParams(
  title: string,
  description: string
): { title: string; description: string; duration: number } {
  return {
    title: title,
    description: description,
    duration: 1000,
  };
}

export function showErrorParams(
  title: string,
  description: string
): {
  title: string;
  description: string;
  variant: 'destructive';
  duration: number;
} {
  return {
    title: title,
    description: description,
    variant: 'destructive',
    duration: 5000,
  };
}

describe('showSuccessParams', () => {
  it('should return the correct params', () => {
    expect(
      showSuccessParams('Success', 'Alias test-alias successfully created')
    ).toEqual({
      title: 'Success',
      description: 'Alias test-alias successfully created',
      duration: 1000,
    });
  });
});

describe('showErrorParams', () => {
  it('should return the correct params', () => {
    expect(
      showErrorParams('Error', 'Alias test-alias successfully created')
    ).toEqual({
      title: 'Error',
      description: 'Alias test-alias successfully created',
      variant: 'destructive',
      duration: 5000,
    });
  });
});
