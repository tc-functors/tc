
const handler = async(event) => {
    try {
      return 'Success';
    } catch (error) {
        console.error(`Failed to process order: ${error.message}`);
        throw error;
    }
};

module.exports = {
  handler
};
